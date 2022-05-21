/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::iter;

use crate::Instruction;

const STANDARD_TAPE_SIZE: usize = 30_000;

pub fn interpret(instructions: &[Instruction], input: &[u8]) {
    let mut head = 0;
    let mut tape = vec![0u8; STANDARD_TAPE_SIZE];

    let mut input_counter = 0;
    let mut program_counter = 0;

    while let Some(instruction) = instructions.get(program_counter) {
        program_counter += 1;

        match instruction {
            Instruction::Add(amount) => {
                if let Some(cell) = tape.get_mut(head) {
                    *cell = (*cell as i8).wrapping_add(*amount) as u8;
                } else {
                    tape.extend(iter::repeat(0).take(head + 1 - tape.len()));
                    tape[head] = *amount as u8;
                }
            }
            Instruction::Move(amount) => {
                if *amount >= 0 || amount.unsigned_abs() <= program_counter {
                    head = (head as isize).wrapping_add(*amount) as usize;
                } else {
                    return;
                }
            }
            Instruction::Write(amount) => {
                if let Some(cell) = tape.get(head) {
                    let c = *cell as char;

                    for _ in 0..*amount {
                        print!("{}", c);
                    }
                }
            }
            Instruction::Read(amount) => {
                input_counter += *amount;

                let input = input.get(input_counter - 1).copied().unwrap_or_default();

                if let Some(cell) = tape.get_mut(head) {
                    *cell = input;
                } else {
                    tape.extend(iter::repeat(0).take(head + 1 - tape.len()));
                    tape[head] = input;
                }
            }
            Instruction::JumpIfZero { location } => {
                if let Some(cell) = tape.get(head) {
                    if *cell == 0 {
                        program_counter = *location;
                    }
                }
            }
            Instruction::JumpIfNotZero { location } => {
                if let Some(cell) = tape.get(head) {
                    if *cell != 0 {
                        program_counter = *location;
                    }
                }
            }

            Instruction::SetAbsolute(value) => {
                if let Some(cell) = tape.get_mut(head) {
                    *cell = *value as u8;
                } else {
                    tape.extend(iter::repeat(0).take(head + 1 - tape.len()));
                    tape[head] = *value as u8;
                }
            }
            Instruction::AddRelative { offset, amount } => {
                let abs_offset = offset.unsigned_abs();

                let index = if *offset >= 0 {
                    head + abs_offset
                } else if abs_offset <= head {
                    head - abs_offset
                } else {
                    return;
                };

                if let Some(cell) = tape.get_mut(index) {
                    *cell = (*cell as i8).wrapping_add(*amount) as u8;
                } else {
                    tape.extend(iter::repeat(0).take(head + 1 - tape.len()));
                    tape[index] = *amount as u8;
                }
            }
            Instruction::AddVectorMove { stride, vector } => {
                if let Some(cell) = tape.get_mut(head..head + 4) {
                    cell[0] = (cell[0] as i8).wrapping_add(vector[0]) as u8;
                    cell[1] = (cell[1] as i8).wrapping_add(vector[1]) as u8;
                    cell[2] = (cell[2] as i8).wrapping_add(vector[2]) as u8;
                    cell[3] = (cell[3] as i8).wrapping_add(vector[3]) as u8;
                } else {
                    tape.extend(iter::repeat(0).take(head + 4 + 1 - tape.len()));
                    tape[head] = vector[0] as u8;
                    tape[head + 1] = vector[1] as u8;
                    tape[head + 2] = vector[2] as u8;
                    tape[head + 3] = vector[3] as u8;
                }

                let abs_stride = stride.unsigned_abs();

                if *stride >= 0 {
                    head += abs_stride;
                } else if abs_stride <= head {
                    head -= abs_stride;
                } else {
                    return;
                }
            }
            Instruction::MoveRightToZero { increment, stride } => {
                while let Some(value) = tape.get_mut(head) {
                    if *value != 0 {
                        *value = (*value as i8).wrapping_add(*increment) as u8;
                        head += stride;
                    } else {
                        break;
                    }
                }
            }
            Instruction::MoveLeftToZero { increment, stride } => {
                while let Some(value) = tape.get_mut(head) {
                    if *value != 0 {
                        if head >= *stride {
                            *value = (*value as i8).wrapping_add(*increment) as u8;
                            head -= stride;
                        } else {
                            return;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
