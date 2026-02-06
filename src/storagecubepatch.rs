use super::*;

fn inc(ib: &mut u32) -> u32 {
    *ib += 1;
    *ib - 1
}

mod image_cube_direction_to_arrayed;
use image_cube_direction_to_arrayed::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ImageOperation<T> {
    Fetch(T),
    Read(T),
    Write(T),
}

impl<T> ImageOperation<T>
where
    T: Clone + Copy,
{
    fn get(&self) -> T {
        match self {
            ImageOperation::Fetch(v) | ImageOperation::Read(v) | ImageOperation::Write(v) => *v,
        }
    }

    fn image_offset(&self) -> usize {
        match self {
            ImageOperation::Fetch(_) => 3,
            ImageOperation::Read(_) => 3,
            ImageOperation::Write(_) => 1,
        }
    }

    fn coordinate_offset(&self) -> usize {
        match self {
            ImageOperation::Fetch(_) => 4,
            ImageOperation::Read(_) => 4,
            ImageOperation::Write(_) => 2,
        }
    }
}

/// Perform the operation on a `Vec<u32>`.
/// Use [u8_slice_to_u32_vec] to convert a `&[u8]` into a `Vec<u32>`.
/// Does not produce any side effects or corrections.
pub fn storagecubepatch(
    in_spv: &[u32],
    corrections: &mut Option<CorrectionMap>,
) -> Result<Vec<u32>, ()> {
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
    let mut op_type_image_idxs = vec![];
    let mut op_type_pointer_idxs = vec![];
    let mut op_variable_idxs = vec![];
    let mut op_load_idxs = vec![];
    let mut op_type_int_idxs = vec![];
    let mut op_type_bool_idxs = vec![];
    let mut op_type_vector_idxs = vec![];
    let mut op_ext_inst_import_idxs = vec![];
    let mut op_function_parameter_idxs = vec![];
    let mut op_function_call_idxs = vec![];
    let mut op_decorate_idxs = vec![];

    let mut image_operation_idxs = vec![];

    let mut spv_idx = 0;
    while spv_idx < spv.len() {
        let op = spv[spv_idx];
        let word_count = hiword(op);
        let instruction = loword(op);

        match instruction {
            SPV_INSTRUCTION_OP_TYPE_IMAGE => op_type_image_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_POINTER => op_type_pointer_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_VARIABLE => op_variable_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_LOAD => op_load_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_INT => op_type_int_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_BOOL => op_type_bool_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_VECTOR => op_type_vector_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_EXT_INST_IMPORT => op_ext_inst_import_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_FUNCTION_PARAMETER => op_function_parameter_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_FUNCTION_CALL => op_function_call_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_DECORATE => op_decorate_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_IMAGE_FETCH => {
                image_operation_idxs.push(ImageOperation::Fetch(spv_idx))
            }
            SPV_INSTRUCTION_OP_IMAGE_READ => {
                image_operation_idxs.push(ImageOperation::Read(spv_idx))
            }
            SPV_INSTRUCTION_OP_IMAGE_WRITE => {
                image_operation_idxs.push(ImageOperation::Write(spv_idx))
            }
            _ => {}
        }

        spv_idx += word_count as usize;
    }

    if op_type_vector_idxs.is_empty()
        || op_type_image_idxs.is_empty()
        || image_operation_idxs.is_empty()
    {
        return Ok(in_spv.to_vec());
    }

    let header_position = last_of_indices!(
        op_type_int_idxs,
        op_type_bool_idxs,
        op_type_vector_idxs,
        op_type_pointer_idxs
    );

    // 2. Insert Required Types
    let mut header_insert = InstructionInsert {
        previous_spv_idx: header_position.unwrap(),
        instruction: vec![],
    };

    let bool_id = ensure_type_bool(
        &spv,
        &op_type_bool_idxs,
        &mut instruction_bound,
        &mut header_insert.instruction,
    );
    let bool_ptr_id = ensure_type_pointer(
        &spv,
        &op_type_pointer_idxs,
        &mut instruction_bound,
        &mut header_insert.instruction,
        SPV_STORAGE_CLASS_FUNCTION,
        bool_id,
    );
    let int32_id = ensure_type_int(
        &spv,
        &op_type_int_idxs,
        &mut instruction_bound,
        &mut header_insert.instruction,
        32,
        SPV_SIGNEDNESS_SIGNED,
    );
    let int32_ptr_id = ensure_type_pointer(
        &spv,
        &op_type_pointer_idxs,
        &mut instruction_bound,
        &mut header_insert.instruction,
        SPV_STORAGE_CLASS_FUNCTION,
        int32_id,
    );
    let v3int32_id = ensure_type_vector(
        &spv,
        &op_type_vector_idxs,
        &mut instruction_bound,
        &mut header_insert.instruction,
        int32_id,
        3,
    );
    let v3int32_ptr_id = ensure_type_pointer(
        &spv,
        &op_type_pointer_idxs,
        &mut instruction_bound,
        &mut header_insert.instruction,
        SPV_STORAGE_CLASS_FUNCTION,
        v3int32_id,
    );
    let v2int32_id = ensure_type_vector(
        &spv,
        &op_type_vector_idxs,
        &mut instruction_bound,
        &mut header_insert.instruction,
        int32_id,
        2,
    );
    let v2int32_ptr_id = ensure_type_pointer(
        &spv,
        &op_type_pointer_idxs,
        &mut instruction_bound,
        &mut header_insert.instruction,
        SPV_STORAGE_CLASS_FUNCTION,
        v2int32_id,
    );
    let glsl_std_id = ensure_ext_inst_import(
        &spv,
        &op_ext_inst_import_idxs,
        &mut instruction_bound,
        &mut header_insert.instruction,
        |s| s.starts_with("GLSL.std."),
        "GLSL.std.450",
    );

    // We only need to validate the bool_id, ptr_int_id
    let type_inputs = CubeDirectionTypeInputs {
        int_id: int32_id,
        v3int_id: v3int32_id,
        v2int_id: v2int32_id,
        bool_id,
        ptr_v3int_id: v3int32_ptr_id,
        ptr_int_id: int32_ptr_id,
        ptr_bool_id: bool_ptr_id,
        ptr_v2int_id: v2int32_ptr_id,
    };
    let (function_type_id, mut function_type_spv) =
        image_cube_direction_to_arrayed_fn_type(&mut instruction_bound, type_inputs);
    header_insert.instruction.append(&mut function_type_spv);

    // 3. Find / Insert Required Constants
    let (shared_constants, mut constants_spv) =
        image_cube_direction_to_arrayed_constants_spv(&mut instruction_bound, int32_id);
    header_insert.instruction.append(&mut constants_spv);

    // 4. Insert Function Type and Definition
    let (function_id, function_spv) = image_cube_direction_to_arrayed_spv(
        &mut instruction_bound,
        type_inputs,
        function_type_id,
        shared_constants,
        glsl_std_id,
    );
    let mut function_definition_words = function_spv;

    // 5. Find OpTypeImage, change Cube -> 2D
    let type_image_ids = op_type_image_idxs
        .iter()
        .filter_map(|idx| {
            let result_id = spv[idx + 1];
            let dim = spv[idx + 3];
            let arrayed = spv[idx + 5];

            if dim == SPV_DIMENSION_CUBE && arrayed == 1 {
                panic!("imageCubeArray is not supported");
            }

            // imageCube => image2DArray
            new_spv[idx + 3] = SPV_DIMENSION_2D;
            new_spv[idx + 5] = 1;

            (dim == SPV_DIMENSION_CUBE).then_some(result_id)
        })
        .collect::<Vec<_>>();

    // 6. Find OpTypePointer -> OpVariable / OpFunctionParameter -> OpLoad
    let type_pointer_ids = op_type_pointer_idxs
        .iter()
        .filter_map(|idx| {
            let result_id = spv[idx + 1];
            let underlying_type_id = spv[idx + 3];

            type_image_ids
                .contains(&underlying_type_id)
                .then_some(result_id)
        })
        .collect::<Vec<_>>();
    let loadable_ids = op_variable_idxs
        .iter()
        // Yes, offsets 1 and 2 are identical
        .chain(op_function_parameter_idxs.iter())
        .filter_map(|idx| {
            let result_id = spv[idx + 2];
            let result_type_id = spv[idx + 1];
            type_pointer_ids
                .contains(&result_type_id)
                .then_some(result_id)
        })
        .collect::<Vec<_>>();
    let loaded_ids = op_load_idxs
        .iter()
        .filter_map(|idx| {
            let result_id = spv[idx + 2];
            let pointer_id = spv[idx + 3];

            loadable_ids.contains(&pointer_id).then_some(result_id)
        })
        .collect::<Vec<_>>();

    // 7. Find and Patch OpImage{Fetch, Read, Write}
    for operation_with_idx in image_operation_idxs.iter() {
        let op_idx = operation_with_idx.get();
        let op_word_count = hiword(spv[op_idx]) as usize;
        let image_id = spv[op_idx + operation_with_idx.image_offset()];
        let coord_id = spv[op_idx + operation_with_idx.coordinate_offset()];

        if loaded_ids.contains(&image_id) {
            // Inject new temp variable to store coordinate
            // TODO: OPT further reduce the number of temp variables by sharing then within the same functions
            let temp_id = inc(&mut instruction_bound);
            instruction_inserts.push(InstructionInsert {
                previous_spv_idx: get_function_label_index_of_instruction_index(&spv, op_idx),
                instruction: vec![
                    encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
                    type_inputs.ptr_v3int_id,
                    temp_id,
                    SPV_STORAGE_CLASS_FUNCTION,
                ],
            });
            // Store existing coordinate, pass to our function, create new instruction
            let output_id = inc(&mut instruction_bound);
            let mut new_instructions = vec![
                encode_word(3, SPV_INSTRUCTION_OP_STORE),
                temp_id,
                coord_id,
                encode_word(5, SPV_INSTRUCTION_OP_FUNCTION_CALL),
                type_inputs.v3int_id,
                output_id,
                function_id,
                temp_id,
            ];
            let cl = new_instructions.len();
            new_instructions.extend_from_slice(&spv[op_idx..op_idx + op_word_count]);
            new_instructions[cl + operation_with_idx.coordinate_offset()] = output_id;
            instruction_inserts.push(InstructionInsert {
                previous_spv_idx: op_idx,
                instruction: new_instructions,
            });

            new_spv[op_idx..op_idx + op_word_count].fill(encode_word(1, SPV_INSTRUCTION_OP_NOP));
        }
    }

    decorate(DecorateIn {
        spv: &spv,
        instruction_inserts: &mut vec![],
        first_op_deocrate_idx: op_decorate_idxs.first().copied(),
        op_decorate_idxs: &op_decorate_idxs,
        affected_decorations: &loadable_ids
            .iter()
            .map(|id| AffectedDecoration {
                original_res_id: *id,
                new_res_id: *id,
                correction_type: CorrectionType::ConvertStorageCube,
            })
            .collect::<Vec<_>>(),
        corrections,
    });

    // 8. Insert New Instructions
    instruction_inserts.insert(0, header_insert);
    insert_new_instructions(&spv, &mut new_spv, &word_inserts, &instruction_inserts);
    new_spv.append(&mut function_definition_words);

    // 9. Remove Instructions that have been Whited Out.
    prune_noops(&mut new_spv);

    // 10. Write New Header and New Code
    Ok(fuse_final(spv_header, new_spv, instruction_bound))
}
