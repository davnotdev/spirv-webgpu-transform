use super::*;

pub struct CorrectDecorateIn<'a> {
    pub new_spv: &'a mut [u32],
    pub descriptor_sets_to_correct: HashSet<u32>,
}

// Correct descriptor sets whose binding index has been invalidated.
// This should be called after instructions have been inserted.
pub fn correct_decorate(cd_in: CorrectDecorateIn) {
    let CorrectDecorateIn {
        new_spv,
        descriptor_sets_to_correct,
    } = cd_in;
    let mut op_decorate_idxs = vec![];
    let mut d_idx = 0;
    while d_idx < new_spv.len() {
        let op = new_spv[d_idx];
        let word_count = hiword(op);
        let instruction = loword(op);
        if instruction == SPV_INSTRUCTION_OP_DECORATE {
            op_decorate_idxs.push(d_idx);
        }

        d_idx += word_count as usize;
    }

    let set_bindings =
        decorate_map_set_bindings(new_spv, &op_decorate_idxs, &descriptor_sets_to_correct);

    for (_, bindings) in set_bindings {
        // We can assume that our new ~~samplers~~ variables will have a greater instruction ID than the original
        // ~~combined image samplers~~ variables.
        let mut prev_binding = None;
        let mut increment = 0;
        for (d_idx, binding) in bindings {
            if Some(binding as i32) == prev_binding {
                increment += 1;
            }
            new_spv[d_idx + 3] = binding + increment;
            prev_binding = Some(binding as i32);
        }
    }
}

pub fn decorate_map_set_bindings(
    spv: &[u32],
    op_decorate_idxs: &[usize],
    set_filter: &HashSet<u32>,
) -> HashMap<u32, Vec<(usize, u32)>> {
    let mut candidates = HashMap::new();

    for d_idx in op_decorate_idxs {
        let result_id = spv[d_idx + 1];
        let type_ = spv[d_idx + 2];
        let value = spv[d_idx + 3];

        match type_ {
            SPV_DECORATION_DESCRIPTOR_SET if set_filter.contains(&value) => {
                candidates.entry(result_id).or_insert((None, None)).0 = Some(value)
            }
            SPV_DECORATION_BINDING => {
                candidates.entry(result_id).or_insert((None, None)).1 = Some((d_idx, value))
            }
            _ => {}
        }
    }

    let mut result = HashMap::new();
    for &descriptor_set in set_filter {
        let mut bindings = candidates
            .iter()
            .filter_map(|(_, &(maybe_descriptor_set, maybe_binding))| {
                let this_descriptor_set = maybe_descriptor_set?;
                let (binding_idx, this_binding) = maybe_binding?;
                (this_descriptor_set == descriptor_set).then_some((*binding_idx, this_binding))
            })
            .collect::<Vec<_>>();
        bindings.sort_by_cached_key(|&(idx, _)| spv[idx + 1]);
        bindings.sort_by_cached_key(|&(_, binding)| binding);

        result.insert(descriptor_set, bindings);
    }

    result
}
