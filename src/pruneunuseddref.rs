use super::*;

pub fn pruneunuseddref(in_spv: &[u32]) -> Result<Vec<u32>, ()> {
    let spv = in_spv.to_owned();

    let instruction_bound = spv[SPV_HEADER_INSTRUCTION_BOUND_OFFSET];
    let magic_number = spv[SPV_HEADER_MAGIC_NUM_OFFSET];

    let spv_header = spv[0..SPV_HEADER_LENGTH].to_owned();

    assert_eq!(magic_number, SPV_HEADER_MAGIC);

    let spv = spv.into_iter().skip(SPV_HEADER_LENGTH).collect::<Vec<_>>();
    let mut new_spv = spv.clone();

    // 1. Find locations instructions we need
    let mut op_type_pointer_idxs = vec![];
    let mut op_type_image_idxs = vec![];
    let mut op_variable_idxs = vec![];
    let mut op_load_idxs = vec![];
    let mut op_function_parameter_idxs = vec![];
    let mut op_function_call_idxs = vec![];
    let mut op_decorate_idxs = vec![];
    let mut op_name_idxs = vec![];

    let mut op_type_sampler_id_map = HashSet::new();
    let mut op_sampled_image_id_map = HashSet::new();

    let mut spv_idx = 0;
    while spv_idx < spv.len() {
        let op = spv[spv_idx];
        let word_count = hiword(op);
        let instruction = loword(op);

        match instruction {
            SPV_INSTRUCTION_OP_TYPE_POINTER => op_type_pointer_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_IMAGE => op_type_image_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_VARIABLE => op_variable_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_LOAD => op_load_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_FUNCTION_PARAMETER => op_function_parameter_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_FUNCTION_CALL => op_function_call_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_DECORATE => op_decorate_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_NAME => op_name_idxs.push(spv_idx),

            SPV_INSTRUCTION_OP_TYPE_SAMPLER => {
                let result_id = spv[spv_idx + 1];
                op_type_sampler_id_map.insert(result_id);
            }
            SPV_INSTRUCTION_OP_SAMPLED_IMAGE => {
                let image_id = spv[spv_idx + 3];
                let sampler_id = spv[spv_idx + 4];
                op_sampled_image_id_map.insert(image_id);
                op_sampled_image_id_map.insert(sampler_id);
            }
            _ => {}
        }

        spv_idx += word_count as usize;
    }

    // 2. Find all OpTypePointer to OpTypeImage and OpTypeSampler
    let image_type_pointers_map = op_type_pointer_idxs
        .iter()
        .filter_map(|&tp_idx| {
            let result_id = spv[tp_idx + 1];
            let underlying_type_id = spv[tp_idx + 3];
            op_type_image_idxs
                .iter()
                .any(|ti_idx| {
                    let type_id = spv[ti_idx + 1];
                    let image_sampled = spv[ti_idx + 7];

                    // `!= 2` filters for storage textures which shouldn't be pruned. 
                    image_sampled != 2 && type_id == underlying_type_id
                })
                .then_some(result_id)
        })
        .collect::<HashSet<_>>();
    let sampler_type_pointers_map = op_type_pointer_idxs
        .iter()
        .filter_map(|&tp_idx| {
            let result_id = spv[tp_idx + 1];
            let underlying_type_id = spv[tp_idx + 3];
            op_type_sampler_id_map
                .contains(&underlying_type_id)
                .then_some(result_id)
        })
        .collect::<HashSet<_>>();

    // 3. Final all OpVariable to OpTypePointers
    let variable_result_map = op_variable_idxs
        .iter()
        .filter_map(|&idx| {
            let tp_id = spv[idx + 1];
            let result_id = spv[idx + 2];

            (image_type_pointers_map.contains(&tp_id) || sampler_type_pointers_map.contains(&tp_id))
                .then_some((result_id, idx))
        })
        .collect::<HashMap<_, _>>();

    // 4. Find all OpLoad to OpSampledImage
    let loaded_idxs = op_load_idxs
        .iter()
        .filter(|&idx| {
            let result_id = spv[idx + 2];
            op_sampled_image_id_map.contains(&result_id)
        })
        .collect::<Vec<_>>();

    let mut used_variable_idxs = loaded_idxs
        .iter()
        .filter_map(|&&load_idx| {
            let pointer = spv[load_idx + 3];
            variable_result_map
                .contains_key(&pointer)
                .then_some(pointer)
        })
        .collect::<HashSet<_>>();

    // 5. Final all OpFunctionParameter to OpLoad
    let function_parameter_idxs = op_function_parameter_idxs.iter().filter(|&fp_idx| {
        let result_id = spv[fp_idx + 2];
        op_load_idxs.iter().any(|&l_idx| {
            let pointer = spv[l_idx + 3];
            pointer == result_id
        })
    });

    // 6. Trace Variables from OpFunctionParameter
    for &fp_idx in function_parameter_idxs {
        let entry = get_function_from_parameter(&spv, fp_idx);
        let variables = trace_function_argument_to_variables(TraceFunctionArgumentToVariablesIn {
            spv: &spv,
            op_variable_idxs: &op_variable_idxs,
            op_function_parameter_idxs: &op_function_parameter_idxs,
            op_function_call_idxs: &op_function_call_idxs,
            entry,
            traced_function_call_idxs: &mut vec![],
        });
        for variable_idx in variables {
            let variable_result_id = spv[variable_idx + 2];
            used_variable_idxs.insert(variable_result_id);
        }
    }

    // 8. Remove unused variables
    let unused_variable_idxs = variable_result_map
        .iter()
        .filter_map(|(id, &idx)| (!used_variable_idxs.contains(id)).then_some(idx))
        .collect::<Vec<_>>();

    // 9. Find OpDecorate / OpName to OpVariable
    let unused_decorate_idxs = op_decorate_idxs
        .iter()
        .filter(|&idx| {
            let target = spv[idx + 1];
            unused_variable_idxs.iter().any(|&v_idx| {
                let result_id = spv[v_idx + 2];
                target == result_id
            })
        })
        .copied()
        .collect::<Vec<_>>();

    let unused_name_idxs = op_name_idxs
        .iter()
        .filter(|&idx| {
            let target = spv[idx + 1];
            unused_variable_idxs.iter().any(|&v_idx| {
                let result_id = spv[v_idx + 2];
                target == result_id
            })
        })
        .copied()
        .collect::<Vec<_>>();

    // 9. Remove instructions
    for spv_idx in unused_variable_idxs
        .into_iter()
        .chain(unused_decorate_idxs)
        .chain(unused_name_idxs)
    {
        let op = spv[spv_idx];
        let word_count = hiword(op) as usize;

        new_spv[spv_idx..spv_idx + word_count].fill(encode_word(1, SPV_INSTRUCTION_OP_NOP));
    }

    prune_noops(&mut new_spv);

    // 10. Write New Header and New Code
    Ok(fuse_final(spv_header, new_spv, instruction_bound))
}
