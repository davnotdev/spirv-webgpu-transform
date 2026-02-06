use super::*;

#[macro_export]
macro_rules! last_of_indices {
    ( $( $v:expr ),+ $(,)? ) => {{
        let mut max_val: Option<usize> = None;

        $(
            for &x in $v.iter() {
                max_val = Some(match max_val {
                    Some(current) => current.max(x),
                    None => x,
                });
            }
        )+

        max_val
    }};
}

pub fn ensure_ext_inst_import<F: Fn(&str) -> bool>(
    spv: &[u32],
    op_ext_inst_import_idxs: &[usize],
    instruction_bound: &mut u32,
    header: &mut Vec<u32>,
    filter: F,
    template: &str,
) -> u32 {
    if let Some(idx) = op_ext_inst_import_idxs.iter().find(|&&idx| {
        let word_count = hiword(spv[idx]) as usize;
        let extension = literal_to_string_le(&spv[idx + 2..idx + word_count])
            .expect("Invalid string in OpExtInstImport");
        filter(&extension)
    }) {
        spv[idx + 1]
    } else {
        let mut ext = string_to_literal_le(template);
        let new_id = *instruction_bound;
        *instruction_bound += 1;
        header.append(&mut vec![
            encode_word(2 + ext.len() as u16, SPV_INSTRUCTION_OP_EXT_INST_IMPORT),
            new_id,
        ]);
        header.append(&mut ext);
        new_id
    }
}

pub fn ensure_type_bool(
    spv: &[u32],
    op_type_bool_idxs: &[usize],
    instruction_bound: &mut u32,
    header: &mut Vec<u32>,
) -> u32 {
    if let Some(idx) = op_type_bool_idxs.first() {
        spv[idx + 1]
    } else {
        let new_id = *instruction_bound;
        *instruction_bound += 1;
        header.append(&mut vec![
            encode_word(2, SPV_INSTRUCTION_OP_TYPE_BOOL),
            new_id,
        ]);
        new_id
    }
}

pub fn ensure_type_int(
    spv: &[u32],
    op_type_int_idxs: &[usize],
    instruction_bound: &mut u32,
    header: &mut Vec<u32>,
    template_width: u32,
    template_signedness: u32,
) -> u32 {
    if let Some(idx) = op_type_int_idxs.iter().find(|&&ty_idx| {
        let width = spv[ty_idx + 2];
        let signedness = spv[ty_idx + 3];

        width == template_width && signedness == template_signedness
    }) {
        spv[idx + 1]
    } else {
        let new_id = *instruction_bound;
        *instruction_bound += 1;
        header.append(&mut vec![
            encode_word(4, SPV_INSTRUCTION_OP_TYPE_INT),
            new_id,
            template_width,
            template_signedness,
        ]);
        new_id
    }
}

pub fn ensure_type_vector(
    spv: &[u32],
    op_type_vector_idxs: &[usize],
    instruction_bound: &mut u32,
    header: &mut Vec<u32>,
    template_component_type_id: u32,
    template_component_count: u32,
) -> u32 {
    if let Some(idx) = op_type_vector_idxs.iter().find(|&&ty_idx| {
        let component_type_id = spv[ty_idx + 2];
        let component_count = spv[ty_idx + 3];

        component_type_id == template_component_type_id
            && component_count == template_component_count
    }) {
        spv[idx + 1]
    } else {
        let new_id = *instruction_bound;
        *instruction_bound += 1;
        header.append(&mut vec![
            encode_word(4, SPV_INSTRUCTION_OP_TYPE_VECTOR),
            new_id,
            template_component_type_id,
            template_component_count,
        ]);
        new_id
    }
}

pub fn ensure_type_pointer(
    spv: &[u32],
    op_type_pointer_idxs: &[usize],
    instruction_bound: &mut u32,
    header: &mut Vec<u32>,
    template_storage_class: u32,
    template_underlying_type_id: u32,
) -> u32 {
    if let Some(tp_idx) = op_type_pointer_idxs.iter().find(|&&tp_idx| {
        let storage_class = spv[tp_idx + 2];
        let underlying_type_id = spv[tp_idx + 3];
        storage_class == template_storage_class && template_underlying_type_id == underlying_type_id
    }) {
        spv[tp_idx + 1]
    } else {
        let new_id = *instruction_bound;
        *instruction_bound += 1;
        header.append(&mut vec![
            encode_word(4, SPV_INSTRUCTION_OP_TYPE_POINTER),
            new_id,
            template_storage_class,
            template_underlying_type_id,
        ]);
        new_id
    }
}
