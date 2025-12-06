use super::*;

type LeftRightOutput = (Option<Vec<u32>>, Option<Vec<u32>>);

/// Reflect one set of patched uniforms onto another shader with the same underlying set of
/// uniforms.
/// This is important for ensuring patched vertex and fragment shaders have the same layout.
/// This is because some transformations occur based off what instructions appear, so as a result,
/// the vertex and fragment shader may have a different layout after a set of transformations
pub fn mirrorpatch(
    left_spv: &[u32],
    left_corrections: &mut Option<CorrectionMap>,
    right_spv: &[u32],
    right_corrections: &mut Option<CorrectionMap>,
) -> Result<LeftRightOutput, ()> {
    if left_corrections.is_none() && right_corrections.is_none() {
        return Ok((None, None));
    }

    let mut left_affected_decorations = vec![];
    let mut right_affected_decorations = vec![];

    let mut left_instruction_bound = left_spv[SPV_HEADER_INSTRUCTION_BOUND_OFFSET];
    let mut right_instruction_bound = right_spv[SPV_HEADER_INSTRUCTION_BOUND_OFFSET];

    let left_corrections_map = left_corrections
        .as_ref()
        .map(|correction_map| correction_map.sets.clone())
        .unwrap_or_default();
    let right_corrections_map = right_corrections
        .as_ref()
        .map(|correction_map| correction_map.sets.clone())
        .unwrap_or_default();

    let mut scan_set_idxs = left_corrections_map
        .keys()
        .chain(right_corrections_map.keys())
        .copied()
        .collect::<Vec<_>>();

    scan_set_idxs.dedup();

    for set_idx in scan_set_idxs {
        let left_bindings = left_corrections_map
            .get(&set_idx)
            .cloned()
            .map(|v| v.bindings)
            .unwrap_or_default();
        let right_bindings = right_corrections_map
            .get(&set_idx)
            .cloned()
            .map(|v| v.bindings)
            .unwrap_or_default();

        for (left_binding_idx, l) in left_bindings.iter() {
            let r = right_bindings
                .get(left_binding_idx)
                .cloned()
                .unwrap_or_default();

            push_affected_decorations(
                &mut right_affected_decorations,
                &mut right_instruction_bound,
                set_idx,
                *left_binding_idx,
                l,
                &r,
            );
        }

        for (right_binding_idx, r) in right_bindings.iter() {
            let l = left_bindings
                .get(right_binding_idx)
                .cloned()
                .unwrap_or_default();

            push_affected_decorations(
                &mut left_affected_decorations,
                &mut left_instruction_bound,
                set_idx,
                *right_binding_idx,
                r,
                &l,
            );
        }
    }

    let l = (!left_affected_decorations.is_empty())
        .then(|| {
            patch_spv_decorations(
                left_spv,
                left_corrections,
                left_instruction_bound,
                &left_affected_decorations,
            )
        })
        .transpose()?;
    let r = (!right_affected_decorations.is_empty())
        .then(|| {
            patch_spv_decorations(
                right_spv,
                right_corrections,
                right_instruction_bound,
                &right_affected_decorations,
            )
        })
        .transpose()?;
    Ok((l, r))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NewVariable {
    set: u32,
    binding: u32,
    new_res_id: u32,
    correction_type: CorrectionType,
}

fn patch_spv_decorations(
    in_spv: &[u32],
    corrections: &mut Option<CorrectionMap>,
    new_instruction_bound: u32,
    affected_decorations: &[NewVariable],
) -> Result<Vec<u32>, ()> {
    let spv = in_spv.to_owned();

    let instruction_bound = new_instruction_bound;
    let magic_number = spv[SPV_HEADER_MAGIC_NUM_OFFSET];
    let spv_header = spv[0..SPV_HEADER_LENGTH].to_owned();

    assert_eq!(magic_number, SPV_HEADER_MAGIC);

    let mut instruction_inserts: Vec<InstructionInsert> = vec![];

    let spv = spv.into_iter().skip(SPV_HEADER_LENGTH).collect::<Vec<_>>();
    let mut new_spv = spv.clone();

    // 1. Find locations instructions we need
    let mut op_decorate_idxs = vec![];
    let mut op_variable_idxs = vec![];
    let mut spv_idx = 0;
    while spv_idx < spv.len() {
        let op = spv[spv_idx];
        let word_count = hiword(op);
        let instruction = loword(op);

        if instruction == SPV_INSTRUCTION_OP_DECORATE {
            op_decorate_idxs.push(spv_idx)
        }
        if instruction == SPV_INSTRUCTION_OP_VARIABLE {
            op_variable_idxs.push(spv_idx)
        }

        spv_idx += word_count as usize;
    }
    let first_op_deocrate_idx = op_decorate_idxs.first().copied();

    // 2. Convert and insert new variables
    let mut cached_original_variable_idxs = HashMap::new();
    let affected_decorations = affected_decorations
        .iter()
        .map(|affected| {
            // Given a set binding, find the original variable
            let NewVariable {
                set,
                binding,
                new_res_id,
                correction_type,
            } = *affected;
            let original_variable_idx =
                *if let Some(idx) = cached_original_variable_idxs.get(&(set, binding)) {
                    idx
                } else {
                    let original_variable_id = op_decorate_idxs
                        .iter()
                        .find_map(|&d_idx| {
                            let target_id = spv[d_idx + 1];
                            let decoration_id = spv[d_idx + 2];
                            let decoration_value = spv[d_idx + 3];
                            (decoration_id == SPV_DECORATION_DESCRIPTOR_SET
                                && decoration_value == set
                                && op_decorate_idxs.iter().any(|&idx| {
                                    let binding_target_id = spv[idx + 1];
                                    let decoration_id = spv[idx + 2];
                                    let decoration_value = spv[idx + 3];
                                    decoration_id == SPV_DECORATION_BINDING
                                        && decoration_value == binding
                                        && target_id == binding_target_id
                                }))
                            .then_some(target_id)
                        })
                        .unwrap();
                    let idx = op_variable_idxs
                        .iter()
                        .find(|&idx| spv[idx + 2] == original_variable_id)
                        .unwrap();
                    cached_original_variable_idxs.insert((set, binding), idx);
                    idx
                };

            // Copy the original variable instruction and substitute new variable id
            let original_variable_id = spv[original_variable_idx + 2];
            let mut new_variable = Vec::new();
            let word_count = hiword(spv[original_variable_idx]);
            new_variable.extend_from_slice(
                &spv[original_variable_idx..original_variable_idx + word_count as usize],
            );
            new_variable[2] = new_res_id;
            instruction_inserts.push(InstructionInsert {
                previous_spv_idx: original_variable_idx,
                instruction: new_variable,
            });

            // Convert into affected decoration
            AffectedDecoration {
                original_res_id: original_variable_id,
                new_res_id,
                correction_type,
            }
        })
        .collect::<Vec<_>>();

    // 3. Insert new OpDecorate
    let DecorateOut {
        descriptor_sets_to_correct,
    } = util::decorate(DecorateIn {
        spv: &spv,
        instruction_inserts: &mut instruction_inserts,
        first_op_deocrate_idx,
        op_decorate_idxs: &op_decorate_idxs,
        affected_decorations: &affected_decorations,
        corrections,
    });

    // 4. Insert New Instructions
    insert_new_instructions(&spv, &mut new_spv, &[], &instruction_inserts);

    // 5. Correct OpDecorate Bindings
    util::correct_decorate(CorrectDecorateIn {
        new_spv: &mut new_spv,
        descriptor_sets_to_correct,
    });

    // 6. Remove Instructions that have been Whited Out.
    prune_noops(&mut new_spv);

    // 7. Write New Header and New Code
    Ok(fuse_final(spv_header, new_spv, instruction_bound))
}

fn push_affected_decorations(
    new_variables: &mut Vec<NewVariable>,
    instruction_bound: &mut u32,
    set: u32,
    binding: u32,
    l: &CorrectionBinding,
    r: &CorrectionBinding,
) {
    let mut ll = l
        .corrections
        .iter()
        .map(Some)
        .enumerate()
        .collect::<Vec<_>>();

    for r_correction in r.corrections.iter() {
        let idx_ty = ll
            .iter()
            .find(|(_, correction)| Some(r_correction) == correction.as_ref().copied())
            .copied();
        if let Some((idx, _)) = idx_ty {
            ll[idx].1 = None;
        }
    }

    let mut offset = 0;
    for (_, op) in ll {
        if let Some(correction) = op {
            *instruction_bound += 1;
            let new_res_id = *instruction_bound - 1;
            new_variables.push(NewVariable {
                set,
                binding: binding + offset,
                new_res_id,
                correction_type: *correction,
            });
        } else {
            offset += 1;
        }
    }
}

#[test]
fn test_push_affected_decorations() {
    let l = CorrectionBinding {
        corrections: vec![
            CorrectionType::SplitCombined,
            CorrectionType::SplitDrefRegular,
            CorrectionType::SplitDrefRegular,
            CorrectionType::SplitCombined,
            CorrectionType::SplitDrefComparison,
        ],
    };

    let r = CorrectionBinding {
        corrections: vec![
            CorrectionType::SplitDrefRegular,
            CorrectionType::SplitDrefComparison,
        ],
    };

    let mut affected = vec![];
    push_affected_decorations(&mut affected, &mut 0, 0, 0, &l, &r);
    assert_eq!(
        affected,
        vec![
            NewVariable {
                set: 0,
                binding: 0,
                new_res_id: 0,
                correction_type: CorrectionType::SplitCombined,
            },
            NewVariable {
                set: 0,
                binding: 1,
                new_res_id: 1,
                correction_type: CorrectionType::SplitDrefRegular,
            },
            NewVariable {
                set: 0,
                binding: 1,
                new_res_id: 2,
                correction_type: CorrectionType::SplitCombined,
            },
        ]
    );

    let mut affected = vec![];
    push_affected_decorations(&mut affected, &mut 0, 0, 0, &r, &l);
    assert_eq!(affected, vec![]);
}
