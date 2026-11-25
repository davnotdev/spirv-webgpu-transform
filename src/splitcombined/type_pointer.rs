use super::*;

pub struct TypePointerIn<'a> {
    pub spv: &'a [u32],
    pub new_spv: &'a mut [u32],

    pub op_type_pointer_idxs: &'a [usize],
    pub op_type_sampled_image_idxs: &'a [usize],
}

pub struct TypePointerOut {
    pub underlying_image_id: u32,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TypePointerId(pub u32);

pub fn type_pointer(tp_in: TypePointerIn) -> HashMap<TypePointerId, TypePointerOut> {
    let mut tp_res = HashMap::new();

    let TypePointerIn {
        spv,
        new_spv,
        op_type_pointer_idxs,
        op_type_sampled_image_idxs,
    } = tp_in;

    op_type_pointer_idxs
        .iter()
        .filter_map(|&tp_spv_idx| {
            // - Find OpTypePointers that ref OpTypeSampledImage
            op_type_sampled_image_idxs.iter().find_map(|&ts_spv_idx| {
                (spv[tp_spv_idx + 3] == spv[ts_spv_idx + 1])
                    .then_some((tp_spv_idx, spv[ts_spv_idx + 2]))
            })
        })
        .for_each(|(tp_spv_idx, underlying_image_id)| {
            // - Change combined image sampler type to underlying image type
            new_spv[tp_spv_idx + 3] = underlying_image_id;

            // - Save the OpTypePointer res id for later
            tp_res.insert(
                TypePointerId(spv[tp_spv_idx + 1]),
                TypePointerOut {
                    underlying_image_id,
                },
            );
        });

    tp_res
}
