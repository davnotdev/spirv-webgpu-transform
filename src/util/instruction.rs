use super::*;

// TODO: Implement `map_spirv` which iterates through a &[u32] of instructions.

pub fn get_last_instruction_index(instructions: &[u32]) -> usize {
    let mut last_off = 0;
    let mut idx = 0;
    while idx < instructions.len() {
        last_off = idx;
        idx += hiword(instructions[idx]) as usize;
    }
    last_off
}
