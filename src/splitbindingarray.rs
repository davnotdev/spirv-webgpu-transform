use super::*;

fn inc(ib: &mut u32) -> u32 {
    *ib += 1;
    *ib - 1
}

mod select_template;

use select_template::*;

/// Perform the operation on a `Vec<u32>`.
/// Use [u8_slice_to_u32_vec] to convert a `&[u8]` into a `Vec<u32>`
/// Either update the existing `corrections` or create a new one.
pub fn splitbindingarray(
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

    let mut op_type_int_idxs = vec![];
    let mut op_type_array_idxs = vec![];
    let mut op_type_pointer_idxs = vec![];
    let mut op_variable_idxs = vec![];
    let mut op_access_chain_idxs = vec![];
    let mut op_type_function_idxs = vec![];
    let mut op_function_parameter_idxs = vec![];
    let mut op_function_call_idxs = vec![];
    let mut op_function_end_idxs = vec![];
    let mut op_decorate_idxs = vec![];
    let mut op_name_idxs = vec![];

    // 1. Find locations instructions we need
    let mut spv_idx = 0;
    while spv_idx < spv.len() {
        let op = spv[spv_idx];
        let word_count = hiword(op);
        let instruction = loword(op);

        match instruction {
            SPV_INSTRUCTION_OP_TYPE_INT => op_type_int_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_ARRAY => op_type_array_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_POINTER => op_type_pointer_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_VARIABLE => op_variable_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_ACCESS_CHAIN => op_access_chain_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_FUNCTION => op_type_function_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_FUNCTION_PARAMETER => op_function_parameter_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_FUNCTION_CALL => op_function_call_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_FUNCTION_END => op_function_end_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_DECORATE => op_decorate_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_NAME => op_name_idxs.push(spv_idx),

            _ => {}
        }

        spv_idx += word_count as usize;
    }

    // TODO: Implement for nested arrays.
    for ta_idx in op_type_array_idxs.iter() {
        let ta_underlying_id = spv[ta_idx + 2];
        for ta_jdx in op_type_array_idxs.iter() {
            if spv[ta_jdx + 2] == ta_underlying_id && ta_idx != ta_jdx {
                unimplemented!("How dare you use nested arrays! (Unimplemented)");
            }
        }
    }

    // 2. OpTypeArray -> OpTypePointer -> OpVariable
    let array_tp_ta_idxs = op_type_pointer_idxs
        .iter()
        .filter_map(|&tp_idx| {
            let tp_storage_class = spv[tp_idx + 2];
            let tp_underlying_id = spv[tp_idx + 3];

            if tp_storage_class != SPV_STORAGE_CLASS_UNIFORM_CONSTANT
                && tp_storage_class != SPV_STORAGE_CLASS_UNIFORM
            {
                return None;
            }

            op_type_array_idxs
                .iter()
                .find(|&ta_idx| {
                    let ta_res_id = spv[ta_idx + 1];

                    ta_res_id == tp_underlying_id
                })
                .map(|&ta_idx| (tp_idx, ta_idx))
        })
        .collect::<Vec<_>>();

    let array_v_ta_idxs = op_variable_idxs
        .iter()
        .filter_map(|&v_idx| {
            let variable_type_id = spv[v_idx + 1];
            array_tp_ta_idxs
                .iter()
                .find(|&(tp_idx, _)| {
                    let tp_res_id = spv[tp_idx + 1];
                    tp_res_id == variable_type_id
                })
                .map(|&(_, ta_idx)| (v_idx, ta_idx))
        })
        .collect::<Vec<_>>();

    // 3. Unroll array variables
    let types_header_position = last_of_indices!(op_type_int_idxs, op_type_pointer_idxs);
    let mut types_header_insert = InstructionInsert {
        previous_spv_idx: types_header_position.unwrap(),
        instruction: vec![],
    };
    let mut new_variables_map = HashMap::new();
    let mut affected_decorations = vec![];

    for &(v_idx, ta_idx) in array_v_ta_idxs.iter() {
        new_spv[v_idx..v_idx + hiword(spv[v_idx]) as usize]
            .fill(encode_word(1, SPV_INSTRUCTION_OP_NOP));

        let mut new_instruction = vec![];

        let underlying_type_id = spv[ta_idx + 2];
        let type_pointer_id = ensure_type_pointer(
            &spv,
            &op_type_pointer_idxs,
            &mut instruction_bound,
            &mut new_instruction,
            SPV_STORAGE_CLASS_UNIFORM_CONSTANT,
            underlying_type_id,
        );
        let length = spv[ta_idx + 3];
        let base_id = instruction_bound;
        instruction_bound += length;

        for i in 0..length {
            new_instruction.append(&mut vec![
                encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
                type_pointer_id,
                base_id + i,
                SPV_STORAGE_CLASS_UNIFORM_CONSTANT,
            ]);
        }
        // Ordering issues with this, let's keep it after all other type pointers.
        //
        // instruction_inserts.push(InstructionInsert {
        //     previous_spv_idx: v_idx,
        //     instruction: new_instruction,
        // });
        types_header_insert.instruction.append(&mut new_instruction);
        new_variables_map.insert(v_idx, base_id);

        let old_result_id = spv[v_idx + 2];
        let new_ids = (base_id..base_id + length).collect::<Vec<_>>();
        affected_decorations.push(AffectedDecoration {
            original_res_id: old_result_id,
            new_res_ids: new_ids,
            correction_type: CorrectionType::SplitBindingArray(length),
        });
    }

    // 4. Create selection functions
    // Ordering issues, so let's just put the function body as the very end.
    let fn_body_header_position = last_of_indices!(
        // op_type_int_idxs,
        // op_type_pointer_idxs,
        // op_variable_idxs,
        op_function_end_idxs
    );
    let mut fn_body_header_insert = InstructionInsert {
        previous_spv_idx: fn_body_header_position.unwrap(),
        instruction: vec![],
    };
    let uint32_id = ensure_type_int(
        &spv,
        &op_type_int_idxs,
        &mut instruction_bound,
        &mut types_header_insert.instruction,
        32,
        SPV_SIGNEDNESS_UNSIGNED,
    );

    let mut selection_map = HashMap::new();
    for (_, ta) in array_tp_ta_idxs.iter() {
        let underlying_type_id = spv[ta + 2];
        let length = spv[ta + 3];

        if let std::collections::hash_map::Entry::Vacant(e) =
            selection_map.entry((underlying_type_id, length))
        {
            let type_pointer_id = ensure_type_pointer(
                &spv,
                &op_type_pointer_idxs,
                &mut instruction_bound,
                &mut types_header_insert.instruction,
                SPV_STORAGE_CLASS_UNIFORM_CONSTANT,
                underlying_type_id,
            );
            println!("GG {:?}", type_pointer_id);

            let inputs = SelectTemplateFunctionInputs {
                uint32_id,
                item_type_id: type_pointer_id,
            };
            let (fn_type_id, mut fn_type_spv) =
                select_template_type_spv(&mut instruction_bound, inputs, length as usize);
            let (fn_id, mut fn_spv) =
                select_template_spv(&mut instruction_bound, inputs, fn_type_id, length as usize);
            types_header_insert.instruction.append(&mut fn_type_spv);
            fn_body_header_insert.instruction.append(&mut fn_spv);
            e.insert(fn_id);
        }
    }

    // 5. Replace OpAccessChain with selection function
    for (ac_idx, v_idx, ta_idx) in op_access_chain_idxs.iter().filter_map(|&ac_idx| {
        let base_id = spv[ac_idx + 3];
        array_v_ta_idxs
            .iter()
            .find(|&(v_idx, _)| {
                let result_id = spv[*v_idx + 2];
                result_id == base_id
            })
            .map(|(v_idx, ta_idx)| (ac_idx, v_idx, ta_idx))
    }) {
        new_spv[ac_idx..ac_idx + hiword(spv[ac_idx]) as usize]
            .fill(encode_word(1, SPV_INSTRUCTION_OP_NOP));

        let old_result_type_id = spv[ac_idx + 1];
        let old_result_id = spv[ac_idx + 2];

        let underlying_type_id = spv[ta_idx + 2];
        let length = spv[ta_idx + 3];
        let fn_id = selection_map[&(underlying_type_id, length)];

        let base_id = new_variables_map[v_idx];

        let mut call = vec![
            encode_word(5 + length as u16, SPV_INSTRUCTION_OP_FUNCTION_CALL),
            old_result_type_id,
            old_result_id,
            fn_id,
            length,
        ];
        call.append(&mut (base_id..base_id + length).collect::<Vec<_>>());
        instruction_inserts.push(InstructionInsert {
            previous_spv_idx: ac_idx,
            instruction: call,
        });
    }

    // 6. Find OpDecorate / OpName to OpVariable
    let unused_decorate_idxs = op_decorate_idxs
        .iter()
        .filter(|&idx| {
            let target = spv[idx + 1];
            new_variables_map.iter().any(|(v_idx, _)| {
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
            new_variables_map.iter().any(|(v_idx, _)| {
                let result_id = spv[v_idx + 2];
                target == result_id
            })
        })
        .copied()
        .collect::<Vec<_>>();

    // 7. Remove Instructions that have been Whited Out.
    for &spv_idx in unused_decorate_idxs.iter().chain(unused_name_idxs.iter()) {
        let op = spv[spv_idx];
        let word_count = hiword(op) as usize;

        new_spv[spv_idx..spv_idx + word_count].fill(encode_word(1, SPV_INSTRUCTION_OP_NOP));
    }

    // 8. OpDecorate
    let DecorateOut {
        descriptor_sets_to_correct,
    } = util::decorate(DecorateIn {
        spv: &spv,
        instruction_inserts: &mut instruction_inserts,
        first_op_deocrate_idx: op_decorate_idxs.first().copied(),
        op_decorate_idxs: &op_decorate_idxs,
        affected_decorations: &affected_decorations,
        corrections,
    });

    // 9. Insert New Instructions
    instruction_inserts.insert(0, types_header_insert);
    instruction_inserts.insert(0, fn_body_header_insert);
    insert_new_instructions(&spv, &mut new_spv, &word_inserts, &instruction_inserts);

    // 10. Correct OpDecorate Bindings
    util::correct_decorate(CorrectDecorateIn {
        new_spv: &mut new_spv,
        descriptor_sets_to_correct,
    });
    prune_noops(&mut new_spv);

    // 11. Write New Header and New Code
    Ok(fuse_final(spv_header, new_spv, instruction_bound))
}
