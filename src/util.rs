use super::*;

mod correct_decorate;
mod decorate;
mod ensure;
mod function;
mod pointer;

pub use correct_decorate::*;
pub use decorate::*;
pub use ensure::*;
pub use function::*;
pub use pointer::*;

pub fn hiword(value: u32) -> u16 {
    ((value >> 16) & 0xFFFF) as u16
}

pub fn loword(value: u32) -> u16 {
    (value & 0xFFFF) as u16
}

pub const fn encode_word(hiword: u16, loword: u16) -> u32 {
    ((hiword as u32) << 16) | (loword as u32)
}

// From spec, "The UTF-8 octets (8-bit bytes) are packed four per word, following the little-endian convention"
pub fn literal_to_string_le(input: &[u32]) -> Result<String, std::str::Utf8Error> {
    let mut bytes = Vec::with_capacity(input.len() * 4);

    for &word in input {
        bytes.extend_from_slice(&word.to_le_bytes());
    }

    str::from_utf8(&bytes).map(|s| s.to_string())
}
pub fn string_to_literal_le(s: &str) -> Vec<u32> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len().div_ceil(4));

    let mut chunk = [0u8; 4];

    for group in bytes.chunks(4) {
        chunk.fill(0);
        chunk[..group.len()].copy_from_slice(group);
        out.push(u32::from_le_bytes(chunk));
    }

    out
}

pub fn insert_new_instructions(
    spv: &[u32],
    new_spv: &mut Vec<u32>,
    word_inserts: &[WordInsert],
    instruction_inserts: &[InstructionInsert],
) {
    // 10. Insert New Instructions
    enum Insert {
        Word(WordInsert),
        Instruction(InstructionInsert),
    }
    let mut inserts = word_inserts
        .iter()
        .cloned()
        .map(Insert::Word)
        .chain(instruction_inserts.iter().cloned().map(Insert::Instruction))
        .collect::<Vec<_>>();

    inserts.sort_by_key(|instruction| match instruction {
        Insert::Word(insert) => insert.idx,
        Insert::Instruction(insert) => insert.previous_spv_idx,
    });
    inserts.iter().rev().for_each(|insert| match insert {
        Insert::Word(new_word) => {
            new_spv.insert(new_word.idx + 1, new_word.word);
            new_spv[new_word.head_idx] = encode_word(
                hiword(new_spv[new_word.head_idx]) + 1,
                loword(new_spv[new_word.head_idx]),
            );
        }
        Insert::Instruction(new_instruction) => {
            let offset = hiword(spv[new_instruction.previous_spv_idx]);
            for idx in 0..new_instruction.instruction.len() {
                new_spv.insert(
                    new_instruction.previous_spv_idx + offset as usize + idx,
                    new_instruction.instruction[idx],
                )
            }
        }
    });
}

pub fn prune_noops(new_spv: &mut Vec<u32>) {
    let mut i_idx = 0;
    while i_idx < new_spv.len() {
        let op = new_spv[i_idx];
        let word_count = hiword(op);
        let instruction = loword(op);

        if instruction == SPV_INSTRUCTION_OP_NOP {
            for _ in 0..word_count {
                new_spv.remove(i_idx);
            }
        } else {
            i_idx += word_count as usize;
        }
    }
}

pub fn fuse_final(
    mut spv_header: Vec<u32>,
    mut new_spv: Vec<u32>,
    new_instruction_bound: u32,
) -> Vec<u32> {
    spv_header[SPV_HEADER_INSTRUCTION_BOUND_OFFSET] = new_instruction_bound;
    let mut out_spv = spv_header;
    out_spv.append(&mut new_spv);
    out_spv
}

#[test]
fn test_literal_string_parsing() {
    let s = "GLSL.std.450".to_owned();
    let u32_vec = string_to_literal_le(&s);
    assert_eq!(u32_vec, vec![1280527431, 1685353262, 808793134]);
    let final_s = literal_to_string_le(&u32_vec);
    assert_eq!(Ok(s), final_s);
}
