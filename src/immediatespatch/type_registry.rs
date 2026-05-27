use super::*;

pub type TypeRegistry = HashMap<u32, Type>;
pub struct BuildTypeRegistryIn<'a> {
    pub spv: &'a [u32],
    pub op_type_float_idxs: &'a [usize],
    pub op_type_int_idxs: &'a [usize],
    pub op_type_vector_idxs: &'a [usize],
    pub op_type_matrix_idxs: &'a [usize],
    pub op_type_array_idxs: &'a [usize],
    pub op_type_struct_idxs: &'a [usize],
    pub op_constant_idxs: &'a [usize],
}

pub fn build_type_registry(build_in: BuildTypeRegistryIn) -> TypeRegistry {
    let BuildTypeRegistryIn {
        spv,
        op_type_float_idxs,
        op_type_int_idxs,
        op_type_vector_idxs,
        op_type_matrix_idxs,
        op_type_array_idxs,
        op_type_struct_idxs,
        op_constant_idxs,
    } = build_in;
    let mut all_idxs = op_type_float_idxs
        .iter()
        .chain(op_type_int_idxs.iter())
        .chain(op_type_vector_idxs.iter())
        .chain(op_type_matrix_idxs.iter())
        .chain(op_type_array_idxs.iter())
        .chain(op_type_struct_idxs.iter())
        .copied()
        .collect::<Vec<_>>();
    // Make sure to walk in order
    all_idxs.sort();

    let mut reg: TypeRegistry = HashMap::new();

    for idx in all_idxs {
        let op = spv[idx];
        let word_count = hiword(op) as usize;
        let instruction = loword(op);
        let id = spv[idx + 1];

        match instruction {
            SPV_INSTRUCTION_OP_TYPE_FLOAT | SPV_INSTRUCTION_OP_TYPE_INT => {
                let width_bytes = spv[idx + 2] / 8;
                reg.insert(
                    id,
                    Type {
                        id,
                        kind: TypeKind::Scalar { width_bytes },
                    },
                );
            }
            SPV_INSTRUCTION_OP_TYPE_VECTOR => {
                let comp_id = spv[idx + 2];
                let count = spv[idx + 3];
                if let Some(component) = reg.get(&comp_id).cloned() {
                    reg.insert(
                        id,
                        Type {
                            id,
                            kind: TypeKind::Vector {
                                component: Box::new(component),
                                count,
                            },
                        },
                    );
                }
            }
            SPV_INSTRUCTION_OP_TYPE_MATRIX => {
                let col_id = spv[idx + 2];
                let cols = spv[idx + 3];
                if let Some(column) = reg.get(&col_id).cloned() {
                    reg.insert(
                        id,
                        Type {
                            id,
                            kind: TypeKind::Matrix {
                                column: Box::new(column),
                                cols,
                            },
                        },
                    );
                }
            }
            SPV_INSTRUCTION_OP_TYPE_ARRAY => {
                let elem_id = spv[idx + 2];
                let len_id = spv[idx + 3];
                let maybe_len = op_constant_idxs.iter().find_map(|&c_idx| {
                    let result_id = spv[c_idx + 2];
                    let literal_value = spv[c_idx + 3];
                    (result_id == len_id).then_some(literal_value)
                });
                if let (Some(element), Some(len)) = (reg.get(&elem_id).cloned(), maybe_len) {
                    reg.insert(
                        id,
                        Type {
                            id,
                            kind: TypeKind::Array {
                                element: Box::new(element),
                                len,
                            },
                        },
                    );
                }
            }
            SPV_INSTRUCTION_OP_TYPE_STRUCT => {
                let mut members = Vec::with_capacity(word_count.saturating_sub(2));
                let mut complete = true;
                for m_word in 2..word_count {
                    let member_id = spv[idx + m_word];
                    if let Some(member) = reg.get(&member_id).cloned() {
                        members.push(member);
                    } else {
                        complete = false;
                        break;
                    }
                }
                if complete {
                    reg.insert(
                        id,
                        Type {
                            id,
                            kind: TypeKind::Struct { members },
                        },
                    );
                }
            }
            _ => {}
        }
    }

    reg
}
