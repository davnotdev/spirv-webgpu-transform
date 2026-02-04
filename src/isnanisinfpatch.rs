use super::*;

fn inc(ib: &mut u32) -> u32 {
    *ib += 1;
    *ib - 1
}

// Someone should make a rust-spirv dsl macro
mod isnan_isinf;
mod shared;

use isnan_isinf::*;
use shared::*;

/// Perform the operation on a `Vec<u32>`.
/// Use [u8_slice_to_u32_vec] to convert a `&[u8]` into a `Vec<u32>`.
/// Does not produce any side effects or corrections.
pub fn isnanisinfpatch(in_spv: &[u32]) -> Result<Vec<u32>, ()> {
    let spv = in_spv.to_owned();

    let mut instruction_bound = spv[SPV_HEADER_INSTRUCTION_BOUND_OFFSET];
    let magic_number = spv[SPV_HEADER_MAGIC_NUM_OFFSET];

    let spv_header = spv[0..SPV_HEADER_LENGTH].to_owned();

    assert_eq!(magic_number, SPV_HEADER_MAGIC);

    let mut instruction_inserts = vec![];
    let word_inserts = vec![];

    let spv = spv.into_iter().skip(SPV_HEADER_LENGTH).collect::<Vec<_>>();
    let mut new_spv = spv.clone();

    // 1. Find locations instructions we need
    let mut op_function_idxs = vec![];
    let mut op_load_idxs = vec![];
    let mut op_type_pointer_idxs = vec![];
    let mut op_is_nan_idxs = vec![];
    let mut op_is_inf_idxs = vec![];
    let mut op_type_bool_idxs = vec![];
    let mut op_type_int_idxs = vec![];
    let mut op_type_float_idxs = vec![];
    let mut op_type_vector_idxs = vec![];

    let mut spv_idx = 0;
    while spv_idx < spv.len() {
        let op = spv[spv_idx];
        let word_count = hiword(op);
        let instruction = loword(op);

        match instruction {
            SPV_INSTRUCTION_OP_FUNCTION => op_function_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_LOAD => op_load_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_POINTER => op_type_pointer_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_IS_NAN => op_is_nan_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_IS_INF => op_is_inf_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_BOOL => op_type_bool_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_INT => op_type_int_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_FLOAT => op_type_float_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_VECTOR => op_type_vector_idxs.push(spv_idx),
            _ => {}
        }

        spv_idx += word_count as usize;
    }

    if op_is_nan_idxs.is_empty() && op_is_inf_idxs.is_empty() {
        return Ok(in_spv.to_vec());
    }
    let Some(last_op_type_pointer) = op_type_pointer_idxs.last() else {
        return Ok(in_spv.to_vec());
    };
    let last_op_type_pointer = *last_op_type_pointer;

    // 2. Useful closures
    let get_float_type_width = |id| {
        op_type_float_idxs
            .iter()
            .find_map(|idx| (spv[idx + 1] == id).then_some(spv[idx + 2]))
    };

    let get_underlying_vector_type = |id| {
        op_type_vector_idxs.iter().find_map(|idx| {
            let result_id = spv[idx + 1];
            let component_type = spv[idx + 2];
            let component_count = spv[idx + 3];

            (result_id == id).then_some((component_type, component_count as usize))
        })
    };

    // 3. Insert shared uint definitions and shared constants
    // Since there are only two main float widths, we will include both for simplicity
    let mut header_insert = InstructionInsert {
        previous_spv_idx: last_op_type_pointer,
        instruction: vec![],
    };

    let ensure_type_int =
        |header: &mut Vec<u32>, instruction_bound: &mut u32, template_width: u32| {
            if let Some(idx) = op_type_int_idxs.iter().find(|&&ty_idx| {
                let width = spv[ty_idx + 2];
                let signedness = spv[ty_idx + 2];

                signedness == SPV_SIGNEDNESS_UNSIGNED && width == template_width
            }) {
                spv[idx + 1]
            } else {
                let new_id = *instruction_bound;
                *instruction_bound += 1;
                header.append(&mut vec![
                    encode_word(4, SPV_INSTRUCTION_OP_TYPE_INT),
                    new_id,
                    template_width,
                    SPV_SIGNEDNESS_UNSIGNED,
                ]);
                new_id
            }
        };
    let ensure_type_ptr = |header: &mut Vec<u32>, instruction_bound: &mut u32, template_id: u32| {
        if let Some(tp_idx) = op_type_pointer_idxs
            .iter()
            .find(|&&tp_idx| template_id == spv[tp_idx + 3])
        {
            spv[tp_idx + 1]
        } else {
            let new_id = *instruction_bound;
            *instruction_bound += 1;
            header.append(&mut vec![
                encode_word(4, SPV_INSTRUCTION_OP_TYPE_POINTER),
                new_id,
                SPV_STORAGE_CLASS_FUNCTION,
                template_id,
            ]);
            new_id
        }
    };

    // NOTE: 64-bit isnan and isinf doesn't actually make much sense, so I will just leave it.
    // In standard glsl, you cannot write that kind of substitution because there is no `uint64_t` nor `doubleBitsToUint`.

    let uint32_id = ensure_type_int(&mut header_insert.instruction, &mut instruction_bound, 32);
    let uint32_ptr_id = ensure_type_ptr(
        &mut header_insert.instruction,
        &mut instruction_bound,
        uint32_id,
    );
    let shared_type_inputs_32 = NanInfSharedTypeInputs {
        uint_id: uint32_id,
        ptr_uint_id: uint32_ptr_id,
    };

    // let uint64_id = ensure_type_int(&mut header_insert.instruction, &mut instruction_bound, 64);
    // let uint64_ptr_id = ensure_type_ptr(
    //     &mut header_insert.instruction,
    //     &mut instruction_bound,
    //     uint64_id,
    // );
    // let shared_type_inputs_64 = NanInfSharedTypeInputs {
    //     uint_id: uint64_id,
    //     ptr_uint_id: uint64_ptr_id,
    // };

    let (shared_constants_32, mut constants_spv_32) =
        nan_inf_shared_constants_spv(&mut instruction_bound, shared_type_inputs_32);
    header_insert.instruction.append(&mut constants_spv_32);
    // let (shared_constants_64, mut constants_spv_64) =
    //     nan_inf_shared_constants(&mut instruction_bound, shared_type_inputs_64);
    // header_insert.instruction.append(&mut constants_spv_64);

    // 4. Insert shared isnan / isinf declaration and definitions

    // SPIR-V doesn't like duplicate `OpTypeFunction`
    let mut fn_type_defs = HashMap::new();
    // We should create the proper number of functions too.
    let mut fn_defs = HashMap::new();

    let mut desc_to_idx: HashMap<_, Vec<usize>> = HashMap::new();
    let fn_set: HashSet<(IsNanOrIsInf, NanInfSharedFunctionInputs, u32, Option<usize>)> =
        op_is_nan_idxs
            .iter()
            .map(|v| (IsNanOrIsInf::IsNan, v))
            .chain(op_is_inf_idxs.iter().map(|v| (IsNanOrIsInf::IsInf, v)))
            .map(|(ty, op_idx)| {
                let input_id = spv[op_idx + 3];

                // We actually cannot rely on loads to get the types of immediate values
                // let load_idx = op_load_idxs
                //     .iter()
                //     .find(|&load_idx| {
                //         let load_result_id = spv[load_idx + 2];
                //         load_result_id == input_id
                //     })
                //     .expect("OpIsNan/Inf not accompanied by OpLoad?");
                // let float_ty_id = spv[load_idx + 1];

                let float_ty_id = trace_previous_intermediate_id(&spv, input_id, *op_idx)
                    .expect("OpIsNan/Inf's argument is not defined?")
                    .result_type_id;
                let original_float_type_id = float_ty_id;
                let (underlying_float_ty_id, float_component_count) =
                    get_underlying_vector_type(float_ty_id)
                        .map(|(a, b)| (a, Some(b)))
                        .unwrap_or((float_ty_id, None));
                let pointer_float_ty_id = op_type_pointer_idxs
                    .iter()
                    .find_map(|&tp_idx| {
                        let result_id = spv[tp_idx + 1];
                        let underlying_type_id = spv[tp_idx + 3];

                        (underlying_type_id == underlying_float_ty_id).then_some(result_id)
                    })
                    .unwrap_or_else(|| {
                        let new_id = instruction_bound;
                        instruction_bound += 1;
                        header_insert.instruction.append(&mut vec![
                            encode_word(4, SPV_INSTRUCTION_OP_TYPE_POINTER),
                            new_id,
                            SPV_STORAGE_CLASS_FUNCTION,
                            underlying_float_ty_id,
                        ]);
                        new_id
                    });
                let bool_ty_id = spv[op_idx + 1];
                let (underlying_bool_ty_id, bool_component_count) =
                    get_underlying_vector_type(bool_ty_id)
                        .map(|(a, b)| (a, Some(b)))
                        .unwrap_or((bool_ty_id, None));
                assert!(bool_component_count == float_component_count);

                let ret = (
                    ty,
                    NanInfSharedFunctionInputs {
                        bool_id: underlying_bool_ty_id,
                        float_id: underlying_float_ty_id,
                        ptr_float_id: pointer_float_ty_id,
                    },
                    original_float_type_id,
                    bool_component_count,
                );
                desc_to_idx.entry(ret).or_default().push(*op_idx);
                ret
            })
            .collect::<HashSet<_, _>>();

    let mut function_definition_words = vec![];

    struct PatchEntry {
        fn_id: u32,
        input: NanInfSharedFunctionInputs,
        original_float_type_id: u32,
        bool_component_count: Option<usize>,
    }
    let mut patch_map: HashMap<usize, PatchEntry> = HashMap::new();
    for (ty, input, original_float_type_id, component_count) in fn_set {
        let (fn_type, mut spv) = nan_inf_fn_type_spv(&mut instruction_bound, input);
        let fn_type = if let Some(existing_fn_type) = fn_type_defs.get(&input).copied() {
            existing_fn_type
        } else {
            header_insert.instruction.append(&mut spv);
            fn_type_defs.insert(input, fn_type);
            fn_type
        };

        let (selected_type_inputs, selected_constants) =
            match get_float_type_width(input.float_id).expect("Our OpTypeFloat dispeared?") {
                32 => (shared_type_inputs_32, shared_constants_32),
                // 64 => (shared_type_inputs_64, shared_constants_64),
                n => panic!(
                    "Float width {} not supported for isnan/isinf substitution",
                    n
                ),
            };

        let (fn_id, mut spv) = is_nan_is_inf_spv(
            &mut instruction_bound,
            ty,
            selected_type_inputs,
            input,
            fn_type,
            selected_constants,
        );
        let fn_id = if let Some(existing_fn_id) = fn_defs.get(&(ty, input, selected_type_inputs)) {
            *existing_fn_id
        } else {
            function_definition_words.append(&mut spv);
            fn_defs.insert((ty, input, selected_type_inputs), fn_id);
            fn_id
        };

        let key = (ty, input, original_float_type_id, component_count);
        for op_idx in &desc_to_idx[&key] {
            patch_map.insert(
                *op_idx,
                PatchEntry {
                    fn_id,
                    input,
                    original_float_type_id,
                    bool_component_count: component_count,
                },
            );
        }
    }

    // 5. Insert additional temp variables and indexing constants for vectored cases
    // We will create the shared data used to generate these variables here
    let mut indexing_constant_instructions = InstructionInsert {
        previous_spv_idx: last_op_type_pointer,
        instruction: vec![],
    };

    let max_components = patch_map
        .values()
        .filter_map(|v| v.bool_component_count)
        .max()
        .unwrap_or(0);

    // We need constants for indexing
    let mut index_ids = vec![];
    for n in 0..max_components {
        let index_id = instruction_bound;
        instruction_bound += 1;
        index_ids.push(index_id);

        indexing_constant_instructions.instruction.append(&mut vec![
            encode_word(4, SPV_INSTRUCTION_OP_CONSTANT),
            uint32_id,
            index_id,
            n as u32,
        ]);
    }

    instruction_inserts.push(indexing_constant_instructions);

    // 6. Insert and patch isnan / isinf usage
    for &op_idx in op_is_nan_idxs.iter().chain(op_is_inf_idxs.iter()) {
        let result_type_id = spv[op_idx + 1];
        let result_id = spv[op_idx + 2];
        let x = spv[op_idx + 3];
        let PatchEntry {
            fn_id,
            input,
            original_float_type_id,
            bool_component_count,
        } = patch_map[&op_idx];

        for i in 0..4 {
            new_spv[op_idx + i] = encode_word(1, SPV_INSTRUCTION_OP_NOP);
        }

        // Both patch implementations need a temp variable
        // TODO: OPT further reduce the number of temp variables by sharing then within the same functions
        let mut temp_variable_instructions = InstructionInsert {
            previous_spv_idx: get_function_label_index_of_instruction_index(&spv, op_idx),
            instruction: vec![],
        };
        let param_id = instruction_bound;
        instruction_bound += 1;
        temp_variable_instructions.instruction.append(&mut vec![
            encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
            input.ptr_float_id,
            param_id,
            SPV_STORAGE_CLASS_FUNCTION,
        ]);

        if let Some(component_count) = bool_component_count {
            let mut new_instructions = InstructionInsert {
                previous_spv_idx: op_idx,
                instruction: vec![],
            };

            // We need a temp variable for the vector itself
            let float_vector_type_pointer_id = op_type_pointer_idxs
                .iter()
                .find_map(|idx| (spv[idx + 3] == original_float_type_id).then_some(spv[idx + 1]))
                .expect("This vector type has no type pointer?");
            let temp_vector_id = instruction_bound;
            instruction_bound += 1;
            temp_variable_instructions.instruction.append(&mut vec![
                encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
                float_vector_type_pointer_id,
                temp_vector_id,
                SPV_STORAGE_CLASS_FUNCTION,
            ]);

            let mut component_results = (0..component_count)
                .map(|n| {
                    let accessed_id = instruction_bound;
                    instruction_bound += 1;
                    let loaded_id = instruction_bound;
                    instruction_bound += 1;
                    let fn_result_id = instruction_bound;
                    instruction_bound += 1;
                    new_instructions.instruction.append(&mut vec![
                        encode_word(3, SPV_INSTRUCTION_OP_STORE),
                        temp_vector_id,
                        x,
                        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
                        input.ptr_float_id,
                        accessed_id,
                        temp_vector_id,
                        index_ids[n],
                        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
                        input.float_id,
                        loaded_id,
                        accessed_id,
                        encode_word(3, SPV_INSTRUCTION_OP_STORE),
                        param_id,
                        loaded_id,
                        encode_word(5, SPV_INSTRUCTION_OP_FUNCTION_CALL),
                        input.bool_id,
                        fn_result_id,
                        fn_id,
                        param_id,
                    ]);
                    fn_result_id
                })
                .collect::<Vec<u32>>();

            new_instructions.instruction.append(&mut vec![
                encode_word(
                    3 + component_count as u16,
                    SPV_INSTRUCTION_OP_COMPOSITE_CONSTRUCT,
                ),
                result_type_id,
                result_id,
            ]);
            new_instructions.instruction.append(&mut component_results);
            instruction_inserts.push(new_instructions);
        } else {
            let new_instructions = InstructionInsert {
                previous_spv_idx: op_idx,
                instruction: vec![
                    encode_word(3, SPV_INSTRUCTION_OP_STORE),
                    param_id,
                    x,
                    encode_word(5, SPV_INSTRUCTION_OP_FUNCTION_CALL),
                    result_type_id,
                    result_id,
                    fn_id,
                    param_id,
                ],
            };
            instruction_inserts.push(new_instructions);
        }

        instruction_inserts.push(temp_variable_instructions);
    }

    // 7. Insert New Instructions
    instruction_inserts.insert(0, header_insert);
    insert_new_instructions(&spv, &mut new_spv, &word_inserts, &instruction_inserts);
    new_spv.append(&mut function_definition_words);

    // 8. Remove Instructions that have been Whited Out.
    prune_noops(&mut new_spv);

    // 9. Write New Header and New Code
    Ok(fuse_final(spv_header, new_spv, instruction_bound))
}
