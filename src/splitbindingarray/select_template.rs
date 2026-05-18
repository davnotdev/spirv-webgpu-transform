use super::*;

// Take the following instructions:
// OpLoad, OpStore, OpAccessChain, OpInBoundsAccessChain, OpCopyMemory
// (i)              (i)            (i)
//
// The instruction's source, `[idx+2]` is replaced with `%base_id+N`
// and duplicated for each case of the index, see template below.
//
pub(super) fn select_template_spv(
    ib: &mut u32,
    index_id: u32,
    base_id: u32,
    original_instruction: &[u32],
    length: usize,
) -> Vec<u32> {
    //
    //  TODO: You can probably decrease the instruction count with OpPhi or OpSelect.
    //
    //              OpSelectionMerge %merge None
    //              OpSwitch %index_id %default %merge 0 %case_0 1 %case_1 ... N %case_N
    //    %case_0 = OpLabel
    //    %temp_0 = {INSTRUCTION}(%base_id+0)
    //              OpBranch %merge
    //    %case_1 = OpLabel
    //    %temp_1 = {INSTRUCTION}(%base_id+1)
    //              OpBranch %merge
    //              ...
    //    %case_N = OpLabel
    //    %temp_N = {INSTRUCTION}(%base_id+N)
    //              OpBranch %merge
    //   %default = OpLabel
    //              {INSTRUCTION}(%base_id+0)
    //     %merge = OpLabel
    //
    // ; Only if there will be a result value.
    // %target_id = OpPhi %underlying_type_id %temp_0 %case_0 %temp_1 %case_1 ... %temp_N %case_N
    //
    //

    let returns_result = matches!(
        loword(original_instruction[0]),
        SPV_INSTRUCTION_OP_LOAD
            | SPV_INSTRUCTION_OP_ACCESS_CHAIN
            | SPV_INSTRUCTION_OP_IN_BOUNDS_ACCESS_CHAIN
    );

    let case_labels = (0..length).map(|_| inc(ib)).collect::<Vec<u32>>();
    let default_label = inc(ib);
    let merge_label = inc(ib);

    let case_temp_ids: Vec<u32> = if returns_result {
        (0..length).map(|_| inc(ib)).collect()
    } else {
        vec![]
    };
    let default_temp_id = if returns_result { inc(ib) } else { 0 };

    let make_case_instruction = |i: usize| {
        let mut patched = original_instruction.to_vec();
        if returns_result {
            patched[2] = case_temp_ids[i];
            patched[3] = base_id + i as u32;
        } else {
            patched[2] = base_id + i as u32;
        }
        patched
    };
    let make_default_instruction = || {
        let mut patched = original_instruction.to_vec();
        if returns_result {
            patched[2] = default_temp_id;
            patched[3] = base_id;
        } else {
            patched[2] = base_id;
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
        spv.extend_from_slice(&make_case_instruction(i));
        spv.extend_from_slice(&[encode_word(2, SPV_INSTRUCTION_OP_BRANCH), merge_label]);
    }

    spv.extend_from_slice(&[encode_word(2, SPV_INSTRUCTION_OP_LABEL), default_label]);
    spv.extend_from_slice(&make_default_instruction());
    spv.extend_from_slice(&[
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
        merge_label,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
        merge_label,
    ]);

    if returns_result {
        let result_type_id = original_instruction[1];
        let original_result_id = original_instruction[2];
        spv.push(encode_word(
            3 + 2 * (length as u16 + 1),
            SPV_INSTRUCTION_OP_PHI,
        ));
        spv.push(result_type_id);
        spv.push(original_result_id);
        for (&case_label, &temp_id) in case_labels.iter().zip(case_temp_ids.iter()) {
            spv.push(temp_id);
            spv.push(case_label);
        }
        spv.push(default_temp_id);
        spv.push(default_label);
    }

    spv
}
