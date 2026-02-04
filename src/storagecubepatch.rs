use super::*;

fn inc(ib: &mut u32) -> u32 {
    *ib += 1;
    *ib - 1
}

mod cube_direction_to_axis;
use cube_direction_to_axis::*;

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
pub fn storagecubepatch(in_spv: &[u32]) -> Result<Vec<u32>, ()> {
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

    // - Find / Insert Required Types
    let type_inputs = CubeDirectionTypeInputs {
        int_id: todo!(),
        v3int_id: todo!(),
        v2int_id: todo!(),
        bool_id: todo!(),
        ptr_v3int_id: todo!(),
        ptr_int_id: todo!(),
        ptr_bool_id: todo!(),
        ptr_v2int_id: todo!(),
    };
    // - Find / Insert Required Constants
    let constant_inputs = CubeDirectionConstants {
        uint_0: todo!(),
        uint_1: todo!(),
        uint_2: todo!(),
        int_0: todo!(),
        int_1: todo!(),
        int_2: todo!(),
        int_3: todo!(),
        int_4: todo!(),
        int_5: todo!(),
    };

    // - Insert Function Type and Definition
    let function_id = 0u32;

    // - Find OpTypeImage, change Cube -> 2D
    let type_image_ids = op_type_image_idxs
        .iter()
        .filter_map(|idx| {
            let result_id = spv[idx + 1];
            let dim = spv[idx + 3];
            let arrayed = spv[idx + 5];

            if dim == SPV_DIMENSION_2D && arrayed == 1 {
                panic!("imageCubeArray is not supported");
            }

            (dim == SPV_DIMENSION_2D).then_some(result_id)
        })
        .collect::<Vec<_>>();

    // - Find OpTypePointer referencing this ^
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

    // - Find OpVariable referencing type pointers
    // - Find OpFunctionParameter referencing type pointers
    let loadable_ids = op_variable_idxs
        .iter()
        .filter_map(|idx| {
            let result_id = spv[idx + 2];
            let result_type_id = spv[idx + 1];
            type_pointer_ids
                .contains(&result_type_id)
                .then_some(result_id)
        })
        .collect::<Vec<_>>();
    // TODO: nested

    // - Find OpLoad referencing loadables
    let loaded_ids = op_load_idxs
        .iter()
        .filter_map(|idx| {
            let result_id = spv[idx + 2];
            let pointer_id = spv[idx + 3];

            loadable_ids.contains(&pointer_id).then_some(result_id)
        })
        .collect::<Vec<_>>();

    // - Find OpFunctionCall referencing loadables
    // TODO: nested

    // - Find OpImage{Fetch, Read, Write} using this ^
    for operation_with_idx in image_operation_idxs.iter() {
        let op_idx = operation_with_idx.get();
        let op_word_count = hiword(spv[op_idx]) as usize;
        let image_id = spv[op_idx + operation_with_idx.image_offset()];
        let coord_id = spv[op_idx + operation_with_idx.coordinate_offset()];

        if loaded_ids.contains(&image_id) {
            // - Inject new temp variable to store coordinate
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
            // - Store existing coordinate, pass to our function, create new instruction
            let output_id = inc(&mut instruction_bound);
            let mut new_instructions = vec![
                encode_word(3, SPV_INSTRUCTION_OP_STORE),
                temp_id,
                coord_id,
                encode_word(3, SPV_INSTRUCTION_OP_FUNCTION_CALL),
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

    // Insert New Instructions
    // instruction_inserts.insert(0, header_insert);
    insert_new_instructions(&spv, &mut new_spv, &word_inserts, &instruction_inserts);
    // new_spv.append(&mut function_definition_words);

    // Remove Instructions that have been Whited Out.
    prune_noops(&mut new_spv);

    // Write New Header and New Code
    Ok(fuse_final(spv_header, new_spv, instruction_bound))
}
