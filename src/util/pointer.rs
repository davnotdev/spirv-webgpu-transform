use super::*;

// This exists only for the sake of punctuality.
#[derive(Debug, Clone, Copy)]
pub struct TracedIntermediate {
    pub idx: usize,
    pub result_type_id: u32,
}

// If we are given an instruction that takes an immediate value, we need a way of knowing its
// underlying type and pointer type.
// That is hard to do because we cannot know instruction this stems from.
// We will assume that some instruction close by follows the form:
// `%result = OpXXXXXX %result_type ...`
// Also, immediate is the incorrect term, but I'll stick to it because it somewhat makes sense
pub fn trace_previous_intermediate_id(
    spv: &[u32],
    id: u32,
    instruction_idx: usize,
) -> Option<TracedIntermediate> {
    let mut spv_idx = 0;
    let mut last_type_id = None;
    while spv_idx < instruction_idx {
        let op = spv[spv_idx];
        let word_count = hiword(op);
        if word_count > 2 && spv[spv_idx + 2] == id {
            last_type_id = Some(TracedIntermediate {
                idx: spv_idx,
                result_type_id: spv[spv_idx + 1],
            })
        }
        spv_idx += word_count as usize;
    }
    last_type_id
}
