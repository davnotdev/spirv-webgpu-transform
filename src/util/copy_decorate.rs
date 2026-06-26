use super::*;

// For storage textures specifically, we want to copy the decorations for cloned variables.
// For the purposes of WebGPU, copying OpDecorate should be sufficient.
// And for WebGPU, only a subset of the following are really needed:
const COPY_DECORATIONS: &[u32] = &[
    SPV_DECORATION_NON_WRITABLE,
    SPV_DECORATION_NON_READABLE,
    SPV_DECORATION_COHERENT,
    SPV_DECORATION_VOLATILE,
    SPV_DECORATION_RESTRICT,
    SPV_DECORATION_ALIASED,
    SPV_DECORATION_RELAXED_PRECISION,
];

pub struct CopyDecorateIn<'a> {
    pub spv: &'a [u32],
    pub op_decorate_idxs: &'a [usize],
    pub instruction_inserts: &'a mut Vec<InstructionInsert>,
    pub old_id: u32,
    pub new_id: u32,
}

pub fn copy_decorate(cd_in: CopyDecorateIn) {
    let CopyDecorateIn {
        spv,
        op_decorate_idxs,
        instruction_inserts,
        old_id,
        new_id,
    } = cd_in;

    let mut first_target_idx = None;

    for &idx in op_decorate_idxs {
        let target_id = spv[idx + 1];
        let decoration = spv[idx + 2];

        if target_id == old_id && COPY_DECORATIONS.contains(&decoration) {
            let word_count = hiword(spv[idx]) as usize;
            let first_target_idx = *first_target_idx.get_or_insert(idx);
            let mut instruction_copy = spv[idx..idx + word_count].to_vec();

            instruction_copy[1] = new_id;

            instruction_inserts.push(InstructionInsert {
                previous_spv_idx: first_target_idx,
                instruction: instruction_copy,
            });
        }
    }
}
