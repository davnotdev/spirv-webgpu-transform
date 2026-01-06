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

enum FloatTypeVariant {
    F32,
    F64,
    F32Vector(usize),
    F64Vector(usize),
}

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
    let mut word_inserts = vec![];

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

    let mut op_type_void_idx = None;

    let mut spv_idx = 0;
    while spv_idx < spv.len() {
        let op = spv[spv_idx];
        let word_count = hiword(op);
        let instruction = loword(op);

        match instruction {
            SPV_INSTRUCTION_OP_TYPE_VOID => op_type_void_idx = Some(spv_idx),
            SPV_INSTRUCTION_OP_FUNCTION => op_function_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_LOAD => op_load_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_POINTER => op_type_pointer_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_IS_NAN => op_is_nan_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_IS_INF => op_is_inf_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_BOOL => op_type_bool_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_INT => op_type_int_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_FLOAT => op_type_float_idxs.push(spv_idx),
            _ => {}
        }

        spv_idx += word_count as usize;
    }

    let op_type_void_idx = op_type_void_idx.expect("Surely an OpTypeVoid is present!");

    if op_is_nan_idxs.is_empty() && op_is_inf_idxs.is_empty() {
        return Ok(in_spv.to_vec());
    }

    // 2. Useful closures
    let get_float_type_width = |id| {
        op_type_int_idxs
            .iter()
            .find_map(|idx| (spv[idx + 1] == id).then_some(spv[idx + 2]))
    };

    let is_bool_vectored = |id| {
        // If id is not a bool, then it is a vector type
        !op_type_bool_idxs.iter().any(|idx| spv[idx + 1] == id)
    };

    // 3. Insert shared uint definitions and shared constants
    // Since there are only two main float widths, we will include both for simplicity
    let mut header_insert = InstructionInsert {
        // A stable place to insert these is at the first OpTypeVoid
        previous_spv_idx: op_type_void_idx,
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
                SPV_STORAGE_CLASS_UNIFORM_FUNCTION,
                template_id,
            ]);
            new_id
        }
    };

    let uint32_id = ensure_type_int(&mut header_insert.instruction, &mut instruction_bound, 32);
    let uint64_id = ensure_type_int(&mut header_insert.instruction, &mut instruction_bound, 64);
    let uint32_ptr_id = ensure_type_ptr(
        &mut header_insert.instruction,
        &mut instruction_bound,
        uint32_id,
    );
    let uint64_ptr_id = ensure_type_ptr(
        &mut header_insert.instruction,
        &mut instruction_bound,
        uint64_id,
    );

    let shared_type_inputs_32 = NanInfSharedTypeInputs {
        uint_id: uint32_id,
        ptr_uint_id: uint32_ptr_id,
    };
    let shared_type_inputs_64 = NanInfSharedTypeInputs {
        uint_id: uint64_id,
        ptr_uint_id: uint64_ptr_id,
    };

    let (shared_constants_32, mut constants_spv_32) =
        nan_inf_shared_constants(&mut instruction_bound, shared_type_inputs_32);
    let (shared_constants_64, mut constants_spv_64) =
        nan_inf_shared_constants(&mut instruction_bound, shared_type_inputs_64);
    header_insert.instruction.append(&mut constants_spv_64);

    // 4. Insert shared isnan / isinf declaration and definitions
    let mut desc_to_idx: HashMap<_, Vec<usize>> = HashMap::new();
    let fn_set = op_is_nan_idxs
        .iter()
        .map(|v| (IsNanOrIsInf::IsNan, v))
        .chain(op_is_inf_idxs.iter().map(|v| (IsNanOrIsInf::IsInf, v)))
        .map(|(ty, op_idx)| {
            let input_id = spv[op_idx + 3];
            let load_idx = op_load_idxs
                .iter()
                .find(|&load_idx| {
                    let load_result_id = spv[load_idx + 2];
                    load_result_id == input_id
                })
                .expect("OpIsNan/Inf not accompanied by OpLoad?");

            let float_ty_id = spv[load_idx + 1];
            let pointer_float_ty_id = op_type_pointer_idxs
                .iter()
                .find_map(|&tp_idx| {
                    let result_id = spv[tp_idx + 1];
                    let underlying_type_id = spv[tp_idx + 3];

                    (underlying_type_id == float_ty_id).then_some(result_id)
                })
                .expect("Loaded float/double while missing pointer type?");
            let bool_ty_id = spv[op_idx + 1];

            let ret = (
                ty,
                NanInfSharedFunctionInputs {
                    bool_id: bool_ty_id,
                    float_id: float_ty_id,
                    ptr_float_id: pointer_float_ty_id,
                },
            );
            desc_to_idx.entry(ret).or_default().push(*op_idx);
            ret
        })
        .collect::<HashMap<_, _>>();

    let mut function_definition_words = vec![];
    let mut patch_map = HashMap::new();
    for (ty, input) in fn_set {
        let (fn_type, mut spv) = nan_inf_fn_type(&mut instruction_bound, input);
        header_insert.instruction.append(&mut spv);

        let selected_constants =
            match get_float_type_width(input.float_id).expect("Our OpTypeFloat dispeared?") {
                32 => shared_constants_32,
                64 => shared_constants_64,
                n => panic!(
                    "Float width {} not supported for isnan/isinf substitution",
                    n
                ),
            };

        let (fn_id, mut spv) = is_nan_is_inf_spv(
            &mut instruction_bound,
            ty,
            shared_type_inputs_32,
            input,
            fn_type,
            selected_constants,
        );
        function_definition_words.append(&mut spv);

        let key = (ty, input);
        for op_idx in &desc_to_idx[&key] {
            *patch_map.entry(op_idx).or_default() = fn_id;
        }
    }

    // 5. Insert and patch isnan

    // 6. Insert and patch isinf

    // 7. Insert New Instructions
    insert_new_instructions(&spv, &mut new_spv, &word_inserts, &instruction_inserts);
    new_spv.append(&mut function_definition_words);

    // 8. Remove Instructions that have been Whited Out.
    prune_noops(&mut new_spv);

    // 9. Write New Header and New Code
    Ok(fuse_final(spv_header, new_spv, instruction_bound))
}
