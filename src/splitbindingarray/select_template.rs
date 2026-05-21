use super::*;

// Take the following instructions:
// OpLoad, OpStore, OpAccessChain, OpInBoundsAccessChain, OpCopyMemory
// (i)              (i)            (i)
//
// The instruction's source, `[idx+2]` is replaced with `%base_id+N`
// and duplicated for each case of the index, see template below.
//
// `flip_store_into` specifically changes `%a = %result` to `%result = %a`
//
pub(super) fn select_template_spv(
    ib: &mut u32,
    base_id: u32,
    index_id: u32,
    switch_instructions: &[u32],
    length: usize,
    flip_store_into: bool,
) -> Vec<u32> {
    //
    //  TODO: You can probably decrease the instruction count with OpPhi or OpSelect.
    //
    //              OpSelectionMerge %merge None
    //              OpSwitch %index_id %default %merge 0 %case_0 1 %case_1 ... N %case_N
    //    %case_0 = OpLabel
    //    %temp_0 = {INSTRUCTION}(%base_id+0)
    //              ...
    //              OpBranch %merge
    //    %case_1 = OpLabel
    //    %temp_1 = {INSTRUCTION}(%base_id+1)
    //              ...
    //              OpBranch %merge
    //              ...
    //    %case_N = OpLabel
    //    %temp_N = {INSTRUCTION}(%base_id+N)
    //              ...
    //              OpBranch %merge
    //   %default = OpLabel
    //    %temp_0 = {INSTRUCTION}(%base_id+0)
    //              ...
    //     %merge = OpLabel
    //
    // ; Only if there will be a result value.
    // %target_id = OpPhi %underlying_type_id %temp_0 %case_0 %temp_1 %case_1 ... %temp_N %case_N
    //

    let mut instruction_offsets = vec![];
    let mut idx = 0;
    while idx < switch_instructions.len() {
        instruction_offsets.push(idx);
        idx += hiword(switch_instructions[idx]) as usize;
    }

    let last_j = instruction_offsets.len() - 1;
    let last_off = instruction_offsets[last_j];
    let returns_result = matches!(
        loword(switch_instructions[last_off]),
        SPV_INSTRUCTION_OP_LOAD
            | SPV_INSTRUCTION_OP_ACCESS_CHAIN
            | SPV_INSTRUCTION_OP_IN_BOUNDS_ACCESS_CHAIN
    );

    let case_labels = (0..length).map(|_| inc(ib)).collect::<Vec<u32>>();
    let default_label = inc(ib);
    let merge_label = inc(ib);

    let case_temps: Vec<Vec<u32>> = (0..length)
        .map(|_| {
            instruction_offsets
                .iter()
                .enumerate()
                .map(|(j, _)| {
                    if j < last_j || returns_result {
                        inc(ib)
                    } else {
                        0
                    }
                })
                .collect()
        })
        .collect();
    let default_temps: Vec<u32> = instruction_offsets
        .iter()
        .enumerate()
        .map(|(j, _)| {
            if j < last_j || returns_result {
                inc(ib)
            } else {
                0
            }
        })
        .collect();

    let make_instructions = |temps: &[u32], base: u32| -> Vec<u32> {
        let mut patched = switch_instructions.to_vec();
        let mut current_source = base;
        for (j, &off) in instruction_offsets.iter().enumerate() {
            if j < last_j || returns_result {
                patched[off + 2] = temps[j];
                patched[off + 3] = current_source;
                current_source = temps[j];
            } else if flip_store_into {
                patched[off + 2] = current_source;
            } else {
                patched[off + 1] = current_source;
            }
        }
        patched
    };

    let mut spv = vec![];

    spv.extend_from_slice(&[
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
        merge_label,
        SPV_SELECTION_CONTROL_NONE,
        encode_word(3 + 2 * length as u16, SPV_INSTRUCTION_OP_SWITCH),
        index_id,
        default_label,
    ]);
    for (i, &case_label) in case_labels.iter().enumerate() {
        spv.push(i as u32);
        spv.push(case_label);
    }

    for (i, &case_label) in case_labels.iter().enumerate() {
        spv.extend_from_slice(&[encode_word(2, SPV_INSTRUCTION_OP_LABEL), case_label]);
        spv.extend_from_slice(&make_instructions(&case_temps[i], base_id + i as u32));
        spv.extend_from_slice(&[encode_word(2, SPV_INSTRUCTION_OP_BRANCH), merge_label]);
    }

    spv.extend_from_slice(&[encode_word(2, SPV_INSTRUCTION_OP_LABEL), default_label]);
    spv.extend_from_slice(&make_instructions(&default_temps, base_id));
    spv.extend_from_slice(&[
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
        merge_label,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
        merge_label,
    ]);

    if returns_result {
        let result_type_id = switch_instructions[last_off + 1];
        let result_id = switch_instructions[last_off + 2];
        spv.push(encode_word(
            3 + 2 * (length as u16 + 1),
            SPV_INSTRUCTION_OP_PHI,
        ));
        spv.push(result_type_id);
        spv.push(result_id);
        for (i, &case_label) in case_labels.iter().enumerate() {
            spv.push(case_temps[i][last_j]);
            spv.push(case_label);
        }
        spv.push(default_temps[last_j]);
        spv.push(default_label);
    }

    spv
}
