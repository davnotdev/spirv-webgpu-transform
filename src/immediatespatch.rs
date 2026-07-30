use super::*;

mod layout;
mod type_registry;

use layout::*;
use type_registry::*;

/// Use [u8_slice_to_u32_vec] to convert a `&[u8]` into a `Vec<u32>`.
pub fn immediatespatch(in_spv: &[u32], corrections: &mut CorrectionMap) -> Result<Vec<u32>, ()> {
    let spv = in_spv.to_owned();

    let instruction_bound = spv[SPV_HEADER_INSTRUCTION_BOUND_OFFSET];
    let magic_number = spv[SPV_HEADER_MAGIC_NUM_OFFSET];

    let spv_header = spv[0..SPV_HEADER_LENGTH].to_owned();

    assert_eq!(magic_number, SPV_HEADER_MAGIC);

    let mut instruction_inserts = vec![];
    let word_inserts = vec![];

    let spv = spv.into_iter().skip(SPV_HEADER_LENGTH).collect::<Vec<_>>();
    let mut new_spv = spv.clone();

    // 1. Find locations of instructions we need
    let mut op_variable_idxs = vec![];
    let mut op_type_pointer_idxs = vec![];
    let mut op_type_struct_idxs = vec![];
    let mut op_type_array_idxs = vec![];
    let mut op_type_matrix_idxs = vec![];
    let mut op_type_vector_idxs = vec![];
    let mut op_type_float_idxs = vec![];
    let mut op_type_int_idxs = vec![];
    let mut op_constant_idxs = vec![];
    let mut op_decorate_idxs = vec![];
    let mut op_member_decorate_idxs = vec![];

    let mut spv_idx = 0;
    while spv_idx < spv.len() {
        let op = spv[spv_idx];
        let word_count = hiword(op);
        let instruction = loword(op);

        match instruction {
            SPV_INSTRUCTION_OP_VARIABLE => op_variable_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_POINTER => op_type_pointer_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_STRUCT => op_type_struct_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_ARRAY => op_type_array_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_MATRIX => op_type_matrix_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_VECTOR => op_type_vector_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_FLOAT => op_type_float_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_INT => op_type_int_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_CONSTANT => op_constant_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_DECORATE => op_decorate_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_MEMBER_DECORATE => op_member_decorate_idxs.push(spv_idx),
            _ => {}
        }

        spv_idx += word_count as usize;
    }

    // 2. Find all `OpVariable` that is a `PushConstant`
    let pc_variables = op_variable_idxs
        .iter()
        .filter_map(|&v_idx| {
            let result_type_id = spv[v_idx + 1];
            let result_id = spv[v_idx + 2];
            let storage_class = spv[v_idx + 3];
            (storage_class == SPV_STORAGE_CLASS_PUSH_CONSTANT).then_some((
                v_idx,
                result_type_id,
                result_id,
            ))
        })
        .collect::<Vec<_>>();

    if pc_variables.is_empty() {
        return Ok(in_spv.to_vec());
    }

    // 3. Find underlying type of variables
    let block_struct_ids = pc_variables
        .iter()
        .map(|&(_, ptr_id, _)| {
            op_type_pointer_idxs
                .iter()
                .find_map(|&tp_idx| {
                    let result_id = spv[tp_idx + 1];
                    let underlying_type_id = spv[tp_idx + 3];
                    (result_id == ptr_id).then_some(underlying_type_id)
                })
                .expect("OpVariable PushConstant referenced an undefined OpTypePointer")
        })
        .collect::<Vec<_>>();

    // 4. Build a registry of every relevant OpType*
    let type_registry = build_type_registry(BuildTypeRegistryIn {
        spv: &spv,
        op_type_float_idxs: &op_type_float_idxs,
        op_type_int_idxs: &op_type_int_idxs,
        op_type_vector_idxs: &op_type_vector_idxs,
        op_type_matrix_idxs: &op_type_matrix_idxs,
        op_type_array_idxs: &op_type_array_idxs,
        op_type_struct_idxs: &op_type_struct_idxs,
        op_constant_idxs: &op_constant_idxs,
    });

    // 5. Rewrite Offset / ArrayStride / MatrixStride decoration
    for &block_struct_id in &block_struct_ids {
        relayout_type_recursive(
            &spv,
            &mut new_spv,
            block_struct_id,
            &type_registry,
            &op_decorate_idxs,
            &op_member_decorate_idxs,
        );
    }

    // 6. Correct OpTypePointer and OpVariable PushConstant -> Uniform
    // TODO: I believe having two of the same OpTypePointer is a validation error
    for &tp_idx in &op_type_pointer_idxs {
        let storage_class = spv[tp_idx + 2];
        if storage_class == SPV_STORAGE_CLASS_PUSH_CONSTANT {
            new_spv[tp_idx + 2] = SPV_STORAGE_CLASS_UNIFORM;
        }
    }
    for &(v_idx, _, _) in &pc_variables {
        new_spv[v_idx + 3] = SPV_STORAGE_CLASS_UNIFORM;
    }

    // 7. Place new uniforms in the set after the last set.
    let first_op_decorate_idx = op_decorate_idxs.first().copied();
    // TODO: What if our `immediates_set` value is faulty?
    let get_max_set = || {
        op_decorate_idxs
            .iter()
            .filter_map(|&d_idx| {
                let decoration_id = spv[d_idx + 2];
                let decoration_value = spv[d_idx + 3];
                (decoration_id == SPV_DECORATION_DESCRIPTOR_SET).then_some(decoration_value)
            })
            .max()
            .map(|max| max + 1)
            .unwrap_or(0)
    };
    let target_set = match (
        corrections.immediates_set,
        corrections.immediates_set_mode.unwrap_or_default(),
    ) {
        (Some(set), ImmediatesSetMode::MaxUpTo) => {
            let max_set = get_max_set();
            if max_set < set { max_set } else { set }
        }
        (Some(set), ImmediatesSetMode::MaxPlusOneUpTo) => {
            let max_set = get_max_set();
            if max_set < set { max_set + 1 } else { set }
        }
        (Some(set), ImmediatesSetMode::Absolute) => set,
        (None, _) => get_max_set(),
    };

    let set_bindings =
        decorate_map_set_bindings(&spv, &op_decorate_idxs, &HashSet::from([target_set]));
    let starting_binding = set_bindings
        .get(&target_set)
        .unwrap()
        .last()
        .map(|&(b, _)| b + 1)
        .unwrap_or(0usize);

    for (binding_idx, &(_, _, var_id)) in pc_variables.iter().enumerate() {
        instruction_inserts.push(InstructionInsert {
            previous_spv_idx: first_op_decorate_idx
                .expect("Push constant block has no OpDecorate (missing Block decoration?)"),
            instruction: vec![
                encode_word(4, SPV_INSTRUCTION_OP_DECORATE),
                var_id,
                SPV_DECORATION_DESCRIPTOR_SET,
                target_set,
                encode_word(4, SPV_INSTRUCTION_OP_DECORATE),
                var_id,
                SPV_DECORATION_BINDING,
                (starting_binding + binding_idx) as u32,
            ],
        });
    }

    corrections.immediates_set = Some(target_set);

    // 8. Insert New Instructions
    insert_new_instructions(&spv, &mut new_spv, &word_inserts, &instruction_inserts);

    // 9. Remove Instructions that have been Whited Out.
    prune_noops(&mut new_spv);

    // 10. Write New Header and New Code
    Ok(fuse_final(spv_header, new_spv, instruction_bound))
}

// Recursively patch Offset / ArrayStride / MatrixStride decorations using our type registry.
fn relayout_type_recursive(
    spv: &[u32],
    new_spv: &mut [u32],
    type_id: u32,
    registry: &TypeRegistry,
    op_decorate_idxs: &[usize],
    op_member_decorate_idxs: &[usize],
) {
    let ty = match registry.get(&type_id) {
        Some(t) => t,
        None => return,
    };

    match &ty.kind {
        TypeKind::Struct { members } => {
            let layout = layout_struct(members, LayoutRule::Std140);

            for (i, new_offset) in layout.member_offsets.iter().enumerate() {
                patch_member_decoration_literal(
                    spv,
                    new_spv,
                    op_member_decorate_idxs,
                    type_id,
                    i as u32,
                    SPV_DECORATION_OFFSET,
                    *new_offset,
                );
            }

            for (i, member) in members.iter().enumerate() {
                let matrix_kind = match &member.kind {
                    TypeKind::Matrix { .. } => Some(&member.kind),
                    TypeKind::Array { element, .. } => match &element.kind {
                        TypeKind::Matrix { .. } => Some(&element.kind),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(TypeKind::Matrix { column, .. }) = matrix_kind {
                    let col_count = column_vec_count(column);
                    let scalar_w = column_scalar_width(column);
                    let new_stride = matrix_stride(col_count, scalar_w, LayoutRule::Std140);
                    patch_member_decoration_literal(
                        spv,
                        new_spv,
                        op_member_decorate_idxs,
                        type_id,
                        i as u32,
                        SPV_DECORATION_MATRIX_STRIDE,
                        new_stride,
                    );
                }
                relayout_type_recursive(
                    spv,
                    new_spv,
                    member.id,
                    registry,
                    op_decorate_idxs,
                    op_member_decorate_idxs,
                );
            }
        }

        TypeKind::Array { element, .. } => {
            let new_stride = array_stride(&element.kind, LayoutRule::Std140);
            for &d_idx in op_decorate_idxs {
                let target_id = spv[d_idx + 1];
                let decoration_id = spv[d_idx + 2];
                if target_id == type_id && decoration_id == SPV_DECORATION_ARRAY_STRIDE {
                    new_spv[d_idx + 3] = new_stride;
                }
            }
            // Ensure arrays of arrays and array of structs are updated.
            relayout_type_recursive(
                spv,
                new_spv,
                element.id,
                registry,
                op_decorate_idxs,
                op_member_decorate_idxs,
            );
        }
        // No effect from scalars, vectors, and matrices.
        _ => {}
    }
}

fn patch_member_decoration_literal(
    spv: &[u32],
    new_spv: &mut [u32],
    op_member_decorate_idxs: &[usize],
    target_id: u32,
    member: u32,
    decoration: u32,
    new_value: u32,
) {
    for &md_idx in op_member_decorate_idxs {
        let md_target_id = spv[md_idx + 1];
        let md_member = spv[md_idx + 2];
        let md_decoration = spv[md_idx + 3];
        if md_target_id == target_id && md_member == member && md_decoration == decoration {
            new_spv[md_idx + 4] = new_value;
        }
    }
}
