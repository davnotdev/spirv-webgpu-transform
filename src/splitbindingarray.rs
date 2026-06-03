use super::*;

fn inc(ib: &mut u32) -> u32 {
    *ib += 1;
    *ib - 1
}

// For the purposes of this patch, I consider an OpTypeImage and OpTypeSampler to be opaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OpaqueArrayType;

mod rechain_instructions;
mod select_template;

use rechain_instructions::*;
use select_template::*;

/// Perform the operation on a `Vec<u32>`.
/// Use [u8_slice_to_u32_vec] to convert a `&[u8]` into a `Vec<u32>`
///
/// Assumed SPIR-V properties for this patch:
///
/// TODO:
/// - No nested
/// - No additional capabilities (SparseResidency or ImageQuery)
///
/// wgpu Properties:
///
/// - The only opaque types that can be in an array are `OpTypeImage` and `OpTypeSampler`
///
/// SPIR-V Properties (These should always be true):
/// - No opaque types in structures
/// - All UBOs and SSBO hold a structure and therefore are accessed with `OpAccessChain*` first.
///
pub fn splitbindingarray(in_spv: &[u32], corrections: &mut CorrectionMap) -> Result<Vec<u32>, ()> {
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
    let mut op_type_image_idxs = vec![];
    let mut op_type_sampler_idxs = vec![];
    let mut op_constant_idxs = vec![];
    let mut op_variable_idxs = vec![];
    let mut op_access_chain_idxs = vec![];
    let mut op_in_bounds_access_chain_idxs = vec![];
    let mut op_load_idxs = vec![];
    let mut op_store_idxs = vec![];
    let mut op_copy_memory_idxs = vec![];
    let mut op_type_function_idxs = vec![];
    let mut op_function_parameter_idxs = vec![];
    let mut op_function_call_idxs = vec![];
    let mut op_function_end_idxs = vec![];
    let mut op_decorate_idxs = vec![];
    let mut op_name_idxs = vec![];
    let mut op_sampled_image_idxs = vec![];

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
            SPV_INSTRUCTION_OP_TYPE_IMAGE => op_type_image_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_SAMPLER => op_type_sampler_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_CONSTANT => op_constant_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_VARIABLE => op_variable_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_ACCESS_CHAIN => op_access_chain_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_IN_BOUNDS_ACCESS_CHAIN => {
                op_in_bounds_access_chain_idxs.push(spv_idx)
            }
            SPV_INSTRUCTION_OP_LOAD => op_load_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_STORE => op_store_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_COPY_MEMORY => op_copy_memory_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_FUNCTION => op_type_function_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_FUNCTION_PARAMETER => op_function_parameter_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_FUNCTION_CALL => op_function_call_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_FUNCTION_END => op_function_end_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_DECORATE => op_decorate_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_NAME => op_name_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_SAMPLED_IMAGE => op_sampled_image_idxs.push(spv_idx),

            _ => {}
        }

        spv_idx += word_count as usize;
    }

    // 2. OpTypeArray -> OpTypePointer
    //      -> OpVariable
    //      -> OpFunctionParameter
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
                .map(|&ta_idx| {
                    let array_type = op_type_image_idxs
                        .iter()
                        .chain(op_type_sampler_idxs.iter())
                        .any(|&t_idx| spv[t_idx + 1] == spv[ta_idx + 2])
                        .then_some(OpaqueArrayType);

                    (tp_idx, ta_idx, array_type)
                })
        })
        .collect::<Vec<_>>();

    // Contains ((OpVariable or OpFunctionParameter), OpTypePointer, Option<OpaqueArrayType>)
    // OpVariable is a subtype of OpFunctionParameter over the first three words.
    let array_vfp_ta_idxs = op_variable_idxs
        .iter()
        .chain(op_function_parameter_idxs.iter())
        .filter_map(|&vfp_idx| {
            let variable_type_id = spv[vfp_idx + 1];
            array_tp_ta_idxs
                .iter()
                .find(|&(tp_idx, _, _)| {
                    let tp_res_id = spv[tp_idx + 1];
                    tp_res_id == variable_type_id
                })
                .map(|&(_, ta_idx, array_type)| (vfp_idx, ta_idx, array_type))
        })
        .collect::<Vec<_>>();

    // TODO: Implement for nested arrays.
    for ta_idx in op_type_array_idxs.iter() {
        let ta_underlying_id = spv[ta_idx + 2];
        for (_, ta_jdx, _) in array_tp_ta_idxs.iter() {
            let j_result_id = spv[ta_jdx + 1];
            if j_result_id == ta_underlying_id && ta_idx != ta_jdx {
                unimplemented!("How dare you use nested arrays! (Unimplemented)");
            }
        }
    }

    // 3. Build mapping of lengths
    let length_map = array_vfp_ta_idxs
        .iter()
        .map(|(_, ta_idx, _)| {
            let length_id = spv[ta_idx + 3];
            let Some(length) = op_constant_idxs.iter().find_map(|&constant_idx| {
                (spv[constant_idx + 2] == length_id).then_some(spv[constant_idx + 3])
            }) else {
                panic!("Missing OpConstant")
            };
            (ta_idx, length)
        })
        .collect::<HashMap<_, _>>();

    // 4. Unroll array variables
    let types_header_position = last_of_indices!(op_type_int_idxs, op_type_pointer_idxs);
    let mut types_header_insert = InstructionInsert {
        previous_spv_idx: types_header_position.unwrap(),
        instruction: vec![],
    };
    let mut new_vfp_map = HashMap::new();
    let mut function_type_changes = HashMap::new();
    let mut affected_decorations = vec![];

    for &(vfp_idx, ta_idx, array_type) in array_vfp_ta_idxs.iter() {
        new_spv[vfp_idx..vfp_idx + hiword(spv[vfp_idx]) as usize]
            .fill(encode_word(1, SPV_INSTRUCTION_OP_NOP));

        let mut new_type_instructions = vec![];

        let instruction = loword(spv[vfp_idx]);
        let underlying_type_id = spv[ta_idx + 2];
        let type_pointer_id = ensure_type_pointer(
            &spv,
            &op_type_pointer_idxs,
            &mut instruction_bound,
            &mut new_type_instructions,
            match array_type {
                Some(OpaqueArrayType) => SPV_STORAGE_CLASS_UNIFORM_CONSTANT,
                _ => SPV_STORAGE_CLASS_UNIFORM,
            },
            underlying_type_id,
        );

        let length = length_map[&ta_idx];

        let base_id = instruction_bound;
        instruction_bound += length;

        match instruction {
            SPV_INSTRUCTION_OP_VARIABLE => {
                for i in 0..length {
                    new_type_instructions.append(&mut vec![
                        encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
                        type_pointer_id,
                        base_id + i,
                        match array_type {
                            Some(OpaqueArrayType) => SPV_STORAGE_CLASS_UNIFORM_CONSTANT,
                            _ => SPV_STORAGE_CLASS_UNIFORM,
                        },
                    ]);
                }
                // Ordering issues with this, let's keep it after all other type pointers.
                //
                // instruction_inserts.push(InstructionInsert {
                //     previous_spv_idx: v_idx,
                //     instruction: new_instruction,
                // });
                types_header_insert
                    .instruction
                    .append(&mut new_type_instructions);
                let old_result_id = spv[vfp_idx + 2];

                // We manually correct the base variable to reuse the original decorations.
                // That way, we can output `N-1` correction bindings.
                for &d_idx in op_decorate_idxs.iter() {
                    if spv[d_idx + 1] == old_result_id {
                        new_spv[d_idx + 1] = base_id;
                    }
                }
                for &n_idx in op_name_idxs.iter() {
                    if spv[n_idx + 1] == old_result_id {
                        new_spv[n_idx + 1] = base_id;
                    }
                }

                // We only want `N-1` correction bindings.
                let new_ids = (base_id + 1..base_id + length).collect::<Vec<_>>();
                affected_decorations.push(AffectedDecoration {
                    original_res_id: old_result_id,
                    new_res_ids: new_ids,
                    correction_type: CorrectionType::SplitBindingArray,
                });
            }
            SPV_INSTRUCTION_OP_FUNCTION_PARAMETER => {
                let mut new_param_instructions = vec![];
                for i in 0..length {
                    new_param_instructions.append(&mut vec![
                        encode_word(3, SPV_INSTRUCTION_OP_FUNCTION_PARAMETER),
                        type_pointer_id,
                        base_id + i,
                    ]);
                }
                instruction_inserts.push(InstructionInsert {
                    previous_spv_idx: vfp_idx,
                    instruction: new_param_instructions,
                });

                let entry = get_function_from_parameter(&spv, vfp_idx);
                let function_type_id = spv[entry.function_idx + 4];

                // `entry.parameter_instruction_idx` is the 0-based ordinal of the parameter
                // within the function; step 5 compares it against the loop variable `i`.
                function_type_changes
                    .entry(function_type_id)
                    .or_insert(vec![])
                    .push((entry.parameter_instruction_idx, type_pointer_id, length));
            }
            _ => unreachable!("Expected OpVariable or OpFunctionParameter"),
        };

        new_vfp_map.insert(vfp_idx, (base_id, ta_idx));
    }

    // 5. Change affected OpTypeFunction
    for &tf_idx in op_type_function_idxs.iter() {
        let tf_result_id = spv[tf_idx + 1];

        let Some(changes) = function_type_changes.get(&tf_result_id) else {
            continue;
        };

        let tf_wc = hiword(spv[tf_idx]) as usize;
        let num_params = tf_wc - 3;

        let mut new_params: Vec<u32> = vec![];
        let mut change_i = 0;
        for i in 0..num_params {
            if change_i < changes.len() && changes[change_i].0 == i {
                let (_, type_ptr, length) = changes[change_i];
                for _ in 0..length {
                    new_params.push(type_ptr);
                }
                change_i += 1;
            } else {
                new_params.push(spv[tf_idx + 3 + i]);
            }
        }

        new_spv[tf_idx..tf_idx + tf_wc].fill(encode_word(1, SPV_INSTRUCTION_OP_NOP));

        let new_wc = (3 + new_params.len()) as u16;
        let mut new_tf = vec![
            encode_word(new_wc, SPV_INSTRUCTION_OP_TYPE_FUNCTION),
            tf_result_id,
            spv[tf_idx + 2], // return type (unchanged)
        ];
        new_tf.extend_from_slice(&new_params);
        types_header_insert.instruction.extend_from_slice(&new_tf);
    }

    let access_idxs = op_access_chain_idxs
        .iter()
        .chain(op_in_bounds_access_chain_idxs.iter())
        .filter_map(|&ac_idx| {
            let base_id = spv[ac_idx + 3];
            array_vfp_ta_idxs
                .iter()
                .find(|&(vfp_idx, _, _)| {
                    let result_id = spv[*vfp_idx + 2];
                    result_id == base_id
                })
                .map(|(vfp_idx, ta_idx, array_type)| (ac_idx, vfp_idx, ta_idx, array_type))
        })
        .collect::<Vec<_>>();

    // 6. Trace array samplers into a map
    // Arrayed samplers turn our neat trace tree into a DAG.
    // To keep things simple, we handle samplers separately.
    // See `opaque_trace.rs` for details.
    let mut arrayed_sampler_map = HashMap::new();
    for &(ac_idx, &vfp_idx, ta_idx, &array_type) in access_idxs.iter() {
        let access_result_id = spv[ac_idx + 2];
        if let Some(OpaqueArrayType) = array_type {
            for &load_idx in op_load_idxs.iter() {
                let result_id = spv[load_idx + 2];
                let pointer_id = spv[load_idx + 3];
                if pointer_id == access_result_id {
                    for &sampled_image_idx in op_sampled_image_idxs.iter() {
                        let sampler_id = spv[sampled_image_idx + 4];
                        if sampler_id == result_id {
                            arrayed_sampler_map
                                .insert(sampled_image_idx, (ac_idx, vfp_idx, ta_idx));
                        }
                    }
                }
            }
        }
    }

    // 7. Replace OpAccessChain with selection function
    for &(ac_idx, vfp_idx, ta_idx, array_type) in access_idxs.iter() {
        let ac_word_count = hiword(spv[ac_idx]) as usize;
        new_spv[ac_idx..ac_idx + ac_word_count].fill(encode_word(1, SPV_INSTRUCTION_OP_NOP));

        let old_result_id = spv[ac_idx + 2];
        let index_0_id = spv[ac_idx + 4];

        let length = length_map[&ta_idx];

        let (base_id, _) = new_vfp_map[vfp_idx];

        if let Some(OpaqueArrayType) = *array_type {
            // When both a texture array and a sampler array feed the same OpSampledImage,
            // the texture AC's processing already generates the correct nested switch
            // (outer = texture index, inner = sampler index via `maybe_sampler_array_data`).
            //
            // Detect this by checking whether this AC is already stored as the sampler
            // dimension in `arrayed_sampler_map`.  If so, just NOP its dependent loads
            // (they reference the now-undefined AC result) and skip switch generation.
            let is_inner_sampler_ac = arrayed_sampler_map
                .values()
                .any(|&(map_ac_idx, _, _)| map_ac_idx == ac_idx);
            if is_inner_sampler_ac {
                for &load_idx in op_load_idxs.iter() {
                    if spv[load_idx + 3] == old_result_id {
                        let wc = hiword(spv[load_idx]) as usize;
                        new_spv[load_idx..load_idx + wc]
                            .fill(encode_word(1, SPV_INSTRUCTION_OP_NOP));
                    }
                }
                continue;
            }

            let load_idxs = op_load_idxs
                .iter()
                .filter(|&idx| {
                    let pointer = spv[idx + 3];
                    pointer == old_result_id
                })
                .copied()
                .collect::<Vec<_>>();
            let dependent_traces = trace_loaded_opaques(&spv, &load_idxs);
            for trace in dependent_traces {
                let maybe_sampler_array_data = match trace.next {
                    OpaqueImageOp::Sampled(sampled_image_op) => {
                        arrayed_sampler_map.get(&sampled_image_op.idx)
                    }
                    _ => None,
                };

                let switch_instructions =
                    reconstruct_opaque_trace_and_overwrite(&spv, &mut new_spv, &trace);
                let underlying_type_and_target_id =
                    get_last_instruction_result_type_and_id(&switch_instructions);
                let rotate_image_sampler = matches!(
                    trace.next,
                    OpaqueImageOp::Sampled(SampledImageOp {
                        parent: SampledImageParent::Sampler,
                        ..
                    })
                );
                let rechain_instructions = |ib: &mut u32, target_id: u32| {
                    let (instructions, output) = rechain_instructions_with_target_id(
                        ib,
                        &switch_instructions,
                        target_id,
                        false,
                        rotate_image_sampler,
                    );
                    (instructions, output.map(|(_, id)| id))
                };
                // Track inner merge labels per outer case so we can fix the outer phi after select_template_spv runs.
                // `select_template_spv` puts the outer case labels in the phi, but with a nested inner switch the actual predecessor
                // of the outer merge is the inner merge block, not the outer case block.
                let mut inner_merge_labels: Vec<u32> = vec![];

                let builder = |ib: &mut u32, target_id: u32| {
                    if let Some((sampler_array_ac_idx, sampler_array_v_idx, sampler_array_ta_idx)) =
                        maybe_sampler_array_data
                    {
                        let (sampler_base_id, _) = new_vfp_map[sampler_array_v_idx];
                        let sampler_index_0_id = spv[sampler_array_ac_idx + 4];
                        let sampler_length = length_map[sampler_array_ta_idx] as usize;

                        // Rechain only the image load for this outer case.
                        // switch_instructions = [image_load, OpSampledImage, ...]
                        // target_id is the split image variable for this outer case.
                        let image_load_wc = hiword(switch_instructions[0]) as usize;
                        let (image_load_instrs, image_out) = rechain_instructions_with_target_id(
                            ib,
                            &switch_instructions[..image_load_wc],
                            target_id,
                            false,
                            false,
                        );
                        let (_, new_image_id) =
                            image_out.expect("image load must produce a result");

                        // Locate OpSampledImage and any instructions that follow it.
                        let si_wc = hiword(switch_instructions[image_load_wc]) as usize;
                        let after_si = &switch_instructions[image_load_wc + si_wc..];
                        let sampler_type_id = spv[op_type_sampler_idxs[0] + 1];

                        // Inner builder: per sampler variable j, emit sampler load + OpSampledImage + trailing instructions.
                        // The image load is placed before the inner switch.
                        let inner_builder = |ib: &mut u32, inner_target_id: u32| {
                            let mut instrs = vec![];

                            let new_sampler_result = inc(ib);
                            instrs.extend_from_slice(&[
                                encode_word(4, SPV_INSTRUCTION_OP_LOAD),
                                sampler_type_id,
                                new_sampler_result,
                                inner_target_id,
                            ]);

                            let new_si_result = inc(ib);
                            let mut si_patched =
                                switch_instructions[image_load_wc..image_load_wc + si_wc].to_vec();
                            si_patched[2] = new_si_result;
                            si_patched[3] = new_image_id;
                            si_patched[4] = new_sampler_result;
                            instrs.extend_from_slice(&si_patched);

                            if !after_si.is_empty() {
                                let (chained, output) = rechain_instructions_with_target_id(
                                    ib,
                                    after_si,
                                    new_si_result,
                                    false,
                                    false,
                                );
                                instrs.extend_from_slice(&chained);
                                return (instrs, output.map(|(_, id)| id));
                            }
                            (instrs, Some(new_si_result))
                        };

                        let mut inner_switch = select_template_spv(
                            ib,
                            sampler_base_id,
                            sampler_index_0_id,
                            sampler_length,
                            inner_builder,
                            underlying_type_and_target_id,
                        );

                        // Find the inner merge label (last OpLabel before the inner phi).
                        let phi_idx = get_last_instruction_index(&inner_switch);
                        {
                            let mut idx = 0;
                            let mut label = 0u32;
                            while idx < phi_idx {
                                if loword(inner_switch[idx]) == SPV_INSTRUCTION_OP_LABEL {
                                    label = inner_switch[idx + 1];
                                }
                                idx += hiword(inner_switch[idx]) as usize;
                            }
                            inner_merge_labels.push(label);
                        }

                        // Patch the inner phi's result id to a fresh id for the outer phi.
                        let output_id = (loword(inner_switch[phi_idx]) == SPV_INSTRUCTION_OP_PHI)
                            .then(|| {
                                let new_id = inc(ib);
                                inner_switch[phi_idx + 2] = new_id;
                                new_id
                            });

                        // Emit: image load once for this outer case, then the inner switch.
                        let mut result = image_load_instrs;
                        result.extend_from_slice(&inner_switch);
                        (result, output_id)
                    } else {
                        rechain_instructions(ib, target_id)
                    }
                };

                let mut switch = select_template_spv(
                    &mut instruction_bound,
                    base_id,
                    index_0_id,
                    length as usize,
                    builder,
                    underlying_type_and_target_id,
                );

                // Patch the outer phi's predecessor labels.
                // select_template_spv filled them with the outer case labels,
                // but each outer case now ends at its inner merge block, not at the outer case label.
                // phi layout: [opword, type, result_id, val0, pred0, val1, pred1, ...]
                if !inner_merge_labels.is_empty() {
                    let phi_idx = get_last_instruction_index(&switch);
                    if loword(switch[phi_idx]) == SPV_INSTRUCTION_OP_PHI {
                        for (i, &label) in inner_merge_labels.iter().enumerate() {
                            switch[phi_idx + 4 + 2 * i] = label;
                        }
                    }
                }

                instruction_inserts.push(InstructionInsert {
                    previous_spv_idx: trace.last_result_id(),
                    instruction: switch,
                });
            }
        } else {
            // For concreate types, find all dependent operations afterwards and replace each instruction with index switch
            for &spv_idx in op_load_idxs
                .iter()
                .chain(op_store_idxs.iter())
                .chain(op_access_chain_idxs.iter())
                .chain(op_in_bounds_access_chain_idxs.iter())
                .chain(op_copy_memory_idxs.iter())
            {
                let word_count = hiword(spv[spv_idx]) as usize;
                let instruction = loword(spv[spv_idx]);

                let mut flip_store_into = false;
                let is_dependent = match instruction {
                    SPV_INSTRUCTION_OP_STORE | SPV_INSTRUCTION_OP_COPY_MEMORY => {
                        // We need to handle cases where buffers are stored from and to.
                        let source_id = spv[spv_idx + 1];
                        let dest_id = spv[spv_idx + 2];

                        // OpStore: %result = %a
                        if dest_id == old_result_id {
                            flip_store_into = true;
                        }

                        source_id == old_result_id || dest_id == old_result_id
                    }
                    SPV_INSTRUCTION_OP_LOAD
                    | SPV_INSTRUCTION_OP_ACCESS_CHAIN
                    | SPV_INSTRUCTION_OP_IN_BOUNDS_ACCESS_CHAIN => {
                        let source_id = spv[spv_idx + 3];
                        source_id == old_result_id
                    }
                    _ => unreachable!("Unexpected instruction {} while matching", instruction),
                };

                if is_dependent && ac_idx != spv_idx {
                    if instruction == SPV_INSTRUCTION_OP_ACCESS_CHAIN
                        || instruction == SPV_INSTRUCTION_OP_IN_BOUNDS_ACCESS_CHAIN
                    {
                        unimplemented!(
                            "Nested OpAccessChain / OpInBoundsAccessChain on binding array (Unimplemented)"
                        );
                    }

                    // We don't want to fully overwrite the access chain since UBOs and SSBOs
                    // accesses will always be followed by these.
                    let mut new_instructions = [
                        &spv[ac_idx..ac_idx + 4],
                        &spv[ac_idx + 5..ac_idx + ac_word_count],
                        &spv[spv_idx..spv_idx + word_count],
                    ]
                    .concat();
                    new_instructions[0] =
                        encode_word(ac_word_count as u16 - 1, SPV_INSTRUCTION_OP_ACCESS_CHAIN);

                    new_spv[spv_idx..spv_idx + word_count]
                        .fill(encode_word(1, SPV_INSTRUCTION_OP_NOP));

                    let builder = &|ib: &mut u32, target_id: u32| {
                        let (instructions, output) = rechain_instructions_with_target_id(
                            ib,
                            &new_instructions,
                            target_id,
                            flip_store_into,
                            false,
                        );
                        (instructions, output.map(|(_, id)| id))
                    };

                    let underlying_type_and_target_id =
                        get_last_instruction_result_type_and_id(&new_instructions);
                    let switch = select_template_spv(
                        &mut instruction_bound,
                        base_id,
                        index_0_id,
                        length as usize,
                        builder,
                        underlying_type_and_target_id,
                    );
                    instruction_inserts.push(InstructionInsert {
                        previous_spv_idx: spv_idx,
                        instruction: switch,
                    });
                }
            }
        }
    }

    // 8. Replace all OpFunctionCall references of arrayed resources
    let new_vfp_id_map = new_vfp_map
        .iter()
        .map(|(&vfp_idx, &v)| {
            let result_id = spv[vfp_idx + 2];
            (result_id, v)
        })
        .collect::<HashMap<_, _>>();
    for &function_call_idx in op_function_call_idxs.iter() {
        const ARGUMENT_OFFSET: usize = 4;
        let word_count = hiword(spv[function_call_idx]) as usize;
        let mut arguments = vec![];
        for &argument_id in spv
            .iter()
            .take(function_call_idx + word_count)
            .skip(function_call_idx + ARGUMENT_OFFSET)
        {
            if let Some(&(base_id, ta_idx)) = new_vfp_id_map.get(&argument_id) {
                let length = length_map[&ta_idx];
                for i in 0..length {
                    arguments.push(base_id + i);
                }
            } else {
                arguments.push(argument_id)
            }
        }

        if arguments.len() != word_count - ARGUMENT_OFFSET {
            new_spv[function_call_idx..function_call_idx + word_count]
                .fill(encode_word(1, SPV_INSTRUCTION_OP_NOP));
            let new_instruction = [
                &[encode_word(
                    (arguments.len() + ARGUMENT_OFFSET) as u16,
                    SPV_INSTRUCTION_OP_FUNCTION_CALL,
                )],
                &spv[function_call_idx + 1..function_call_idx + ARGUMENT_OFFSET],
                arguments.as_slice(),
            ]
            .concat();
            instruction_inserts.push(InstructionInsert {
                previous_spv_idx: function_call_idx,
                instruction: new_instruction,
            });
        }
    }

    // 9. Find OpDecorate / OpName to OpVariable
    let unused_decorate_idxs = op_decorate_idxs
        .iter()
        .filter(|&&idx| {
            let target = spv[idx + 1];
            if new_spv[idx + 1] != target {
                return false;
            }
            new_vfp_map.iter().any(|(vfp_idx, _)| {
                let result_id = spv[vfp_idx + 2];
                target == result_id
            })
        })
        .copied()
        .collect::<Vec<_>>();
    let unused_name_idxs = op_name_idxs
        .iter()
        .filter(|&&idx| {
            let target = spv[idx + 1];
            if new_spv[idx + 1] != target {
                return false;
            }
            new_vfp_map.iter().any(|(vfp_idx, _)| {
                let result_id = spv[vfp_idx + 2];
                target == result_id
            })
        })
        .copied()
        .collect::<Vec<_>>();

    // 10. Remove Instructions that have been Whited Out.
    for &spv_idx in unused_decorate_idxs.iter().chain(unused_name_idxs.iter()) {
        let op = spv[spv_idx];
        let word_count = hiword(op) as usize;

        new_spv[spv_idx..spv_idx + word_count].fill(encode_word(1, SPV_INSTRUCTION_OP_NOP));
    }

    // 11. OpDecorate
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

    // 12. Insert New Instructions
    instruction_inserts.insert(0, types_header_insert);
    insert_new_instructions(&spv, &mut new_spv, &word_inserts, &instruction_inserts);

    // 13. Correct OpDecorate Bindings
    util::correct_decorate(CorrectDecorateIn {
        new_spv: &mut new_spv,
        descriptor_sets_to_correct,
    });
    prune_noops(&mut new_spv);

    // 14. Write New Header and New Code
    Ok(fuse_final(spv_header, new_spv, instruction_bound))
}
