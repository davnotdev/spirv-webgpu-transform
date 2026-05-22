use super::*;

//
// Take the any chain of instructions with the following form:
// OpSomething %result_type_id %result_id %input ...
//
// New temp variables are properly chained between instructions.
//
// The final instruction's `[idx+2]` is replaced with `%target_id`
//
// The final instruction can be a write operation.
// Write instructions are specially checked for because they follow a different convention.
// The following are considered valid write instructions:
// OpStore, OpCopyMemory, OpImageWrite
//
// `flip_store_into` specifically changes `%a = %result` to `%result = %a`
// `chain_sampler_over_image` specifically changes OpImageSampled to chain to the sampler.
//
pub fn rechain_instructions_with_target_id(
    ib: &mut u32,
    snippet: &[u32],
    target_id: u32,
    flip_store_into: bool,
    rotate_image_sampler: bool,
) -> (Vec<u32>, Option<(u32, u32)>) {
    let mut instruction_offsets = vec![];
    let mut idx = 0;
    while idx < snippet.len() {
        instruction_offsets.push(idx);
        idx += hiword(snippet[idx]) as usize;
    }

    let last_j = instruction_offsets.len() - 1;
    let last_off = instruction_offsets[last_j];
    let returns_result = !matches!(
        loword(snippet[last_off]),
        SPV_INSTRUCTION_OP_STORE | SPV_INSTRUCTION_OP_COPY_MEMORY | SPV_INSTRUCTION_OP_IMAGE_WRITE
    );

    let mut patched = snippet.to_vec();
    let mut current_source = target_id;
    for (j, &off) in instruction_offsets.iter().enumerate() {
        if j < last_j || returns_result {
            let new_temp = inc(ib);

            if rotate_image_sampler && loword(patched[off]) == SPV_INSTRUCTION_OP_SAMPLED_IMAGE {
                patched[off + 4] = current_source;
            } else {
                patched[off + 3] = current_source;
            }

            patched[off + 2] = new_temp;
            current_source = new_temp;
        } else if flip_store_into {
            patched[off + 2] = current_source;
        } else {
            patched[off + 1] = current_source;
        }
    }

    let last_offset = instruction_offsets[last_j];
    let underlying_type_and_target = (patched[last_offset + 1], patched[last_offset + 2]);

    (
        patched,
        returns_result.then_some(underlying_type_and_target),
    )
}

// Intended to run alongside the previous function with the same snippet properties.
// If the last instruction is not an expected store operation, return the final result type and id.
pub fn get_last_instruction_result_type_and_id(snippet: &[u32]) -> Option<(u32, u32)> {
    let mut last_off = 0;
    let mut idx = 0;
    while idx < snippet.len() {
        last_off = idx;
        idx += hiword(snippet[idx]) as usize;
    }

    let returns_result = !matches!(
        loword(snippet[last_off]),
        SPV_INSTRUCTION_OP_STORE | SPV_INSTRUCTION_OP_COPY_MEMORY | SPV_INSTRUCTION_OP_IMAGE_WRITE
    );

    returns_result.then_some((snippet[last_off + 1], snippet[last_off + 2]))
}

#[test]
fn two_chained_result_instructions() {
    #[rustfmt::skip]
        let snippet: &[u32] = &[
            encode_word(4, SPV_INSTRUCTION_OP_LOAD), 10, 1, 2,
            encode_word(4, SPV_INSTRUCTION_OP_LOAD), 20, 3, 4,
        ];
    let mut ib = 500u32;
    let (out, result) = rechain_instructions_with_target_id(&mut ib, snippet, 100, false, false);
    assert_eq!(out[2], 500);
    assert_eq!(out[3], 100);
    assert_eq!(out[6], 501);
    assert_eq!(out[7], 500);
    assert_eq!(result, Some((20, 501)));
}

#[test]
fn terminal_store_no_flip() {
    #[rustfmt::skip]
        let snippet: &[u32] = &[
            encode_word(4, SPV_INSTRUCTION_OP_LOAD),  10, 1, 2,
            encode_word(3, SPV_INSTRUCTION_OP_STORE), 3, 4,
        ];
    let mut ib = 500u32;
    let (out, result) = rechain_instructions_with_target_id(&mut ib, snippet, 100, false, false);
    assert_eq!(out[2], 500);
    assert_eq!(out[3], 100);
    assert_eq!(out[5], 500);
    assert_eq!(out[6], 4);
    assert_eq!(result, None);
}

#[test]
fn terminal_store_flip() {
    #[rustfmt::skip]
        let snippet: &[u32] = &[
            encode_word(4, SPV_INSTRUCTION_OP_LOAD),  10, 1, 2,
            encode_word(3, SPV_INSTRUCTION_OP_STORE), 3, 4,
        ];
    let mut ib = 500u32;
    let (out, result) = rechain_instructions_with_target_id(&mut ib, snippet, 100, true, false);
    assert_eq!(out[2], 500);
    assert_eq!(out[3], 100);
    assert_eq!(out[5], 3);
    assert_eq!(out[6], 500);
    assert_eq!(result, None);
}
