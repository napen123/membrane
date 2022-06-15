/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::mem;

use crate::instruction::Instruction;
use crate::interpreter::TapeSize;

// 2865
// TODO: Improve optimizations by taking the tape size into account.
pub fn optimize(verbose: bool, instructions: &mut Vec<Instruction>, _tape_size: TapeSize) {
    let raw_count = instructions.len();
    let mut buffer = Vec::with_capacity(raw_count);

    if verbose {
        println!("INIT: {} instruction(s)", raw_count);
    }

    loop {
        let start_instruction_count = instructions.len();
        {
            squash_and_clean(instructions, &mut buffer);

            substitute_patterns_4(instructions, &mut buffer);
            substitute_patterns_3(instructions, &mut buffer);
            substitute_patterns_2(instructions, &mut buffer);

            remove_spurious_loops(instructions, &mut buffer);
        }
        let end_instruction_count = instructions.len();

        if verbose {
            println!(
                "PASS: {} instruction(s) [{:.2}% -- decreased by {} instruction(s)]",
                end_instruction_count,
                (end_instruction_count as f32) / (raw_count as f32),
                start_instruction_count - end_instruction_count
            );
        }

        if end_instruction_count >= start_instruction_count {
            break;
        }
    }

    fix_loops(instructions);
}

fn squash_and_clean(instructions: &mut Vec<Instruction>, buffer: &mut Vec<Instruction>) {
    {
        let mut iterator = instructions.drain(..).peekable();

        while let Some(instruction) = iterator.next() {
            match instruction {
                Instruction::Add(start_amount) => {
                    let mut accumulator = start_amount;

                    while let Some(Instruction::Add(next_amount)) = iterator.peek() {
                        accumulator = accumulator.wrapping_add(*next_amount);
                        iterator.next();
                    }

                    if accumulator != 0 {
                        buffer.push(Instruction::Add(accumulator));
                    }
                }
                Instruction::Move(start_amount) => {
                    let mut accumulator = start_amount;

                    while let Some(Instruction::Move(next_amount)) = iterator.peek() {
                        match accumulator.checked_add(*next_amount) {
                            Some(new_accumulator) => {
                                accumulator = new_accumulator;
                                iterator.next();
                            }
                            None => {
                                break;
                            }
                        }
                    }

                    if accumulator != 0 {
                        buffer.push(Instruction::Move(accumulator));
                    }
                }
                Instruction::Write(start_amount) => {
                    let mut accumulator = start_amount;

                    while let Some(Instruction::Write(next_amount)) = iterator.peek() {
                        match accumulator.checked_add(*next_amount) {
                            Some(new_accumulator) => {
                                accumulator = new_accumulator;
                                iterator.next();
                            }
                            None => {
                                break;
                            }
                        }
                    }

                    if accumulator != 0 {
                        buffer.push(Instruction::Write(accumulator));
                    }
                }
                Instruction::Read(start_amount) => {
                    let mut accumulator = start_amount;

                    while let Some(Instruction::Read(next_amount)) = iterator.peek() {
                        match accumulator.checked_add(*next_amount) {
                            Some(new_accumulator) => {
                                accumulator = new_accumulator;
                                iterator.next();
                            }
                            None => {
                                break;
                            }
                        }
                    }

                    if accumulator != 0 {
                        buffer.push(Instruction::Read(accumulator));
                    }
                }

                Instruction::SetValue(start_value) => {
                    let mut final_value = start_value;

                    while let Some(Instruction::SetValue(next_value)) = iterator.peek() {
                        final_value = *next_value;
                        iterator.next();
                    }

                    buffer.push(Instruction::SetValue(final_value));
                }

                Instruction::AddRelative {
                    offset,
                    amount: start_amount,
                } => {
                    let mut total_amount = start_amount;

                    while let Some(Instruction::AddRelative {
                        offset: next_offset,
                        amount: next_amount,
                    }) = iterator.peek()
                    {
                        if offset == *next_offset {
                            total_amount = total_amount.wrapping_add(*next_amount);
                            iterator.next();
                        } else {
                            break;
                        }
                    }

                    if total_amount != 0 {
                        if offset != 0 {
                            buffer.push(Instruction::AddRelative {
                                offset,
                                amount: total_amount,
                            });
                        } else {
                            buffer.push(Instruction::Add(total_amount));
                        }
                    }
                }
                Instruction::AddVector { vector } => {
                    let mut accumulator = vector;

                    while let Some(Instruction::AddVector {
                        vector: next_vector,
                    }) = iterator.peek()
                    {
                        accumulator[0] = accumulator[0].wrapping_add(next_vector[0]);
                        accumulator[1] = accumulator[1].wrapping_add(next_vector[1]);
                        accumulator[2] = accumulator[2].wrapping_add(next_vector[2]);
                        accumulator[3] = accumulator[3].wrapping_add(next_vector[3]);
                        iterator.next();
                    }

                    match accumulator {
                        [0, 0, 0, 0] => {}
                        [amount, 0, 0, 0] => buffer.push(Instruction::Add(amount)),
                        [0, amount, 0, 0] => {
                            buffer.push(Instruction::AddRelative { offset: 1, amount });
                        }
                        [0, 0, amount, 0] => {
                            buffer.push(Instruction::AddRelative { offset: 2, amount });
                        }
                        [0, 0, 0, amount] => {
                            buffer.push(Instruction::AddRelative { offset: 3, amount });
                        }
                        _ => buffer.push(Instruction::AddVector {
                            vector: accumulator,
                        }),
                    }
                }

                inst @ Instruction::MoveRightToZero { .. }
                | inst @ Instruction::MoveLeftToZero { .. } => {
                    while let Some(
                        Instruction::MoveRightToZero { .. } | Instruction::MoveLeftToZero { .. },
                    ) = iterator.peek()
                    {
                        iterator.next();
                    }

                    buffer.push(inst);
                }

                _ => {
                    buffer.push(instruction);
                }
            }
        }
    }

    mem::swap(instructions, buffer);
}

fn substitute_patterns_2(instructions: &mut Vec<Instruction>, buffer: &mut Vec<Instruction>) {
    if instructions.len() < 2 {
        return;
    }

    let mut matched = false;
    let mut iterator = instructions.windows(2);

    while let Some(window) = iterator.next() {
        matched = false;

        match window {
            [Instruction::Add(_), Instruction::SetValue(value)] => {
                matched = true;
                buffer.push(Instruction::SetValue(*value));
            }
            [Instruction::Add(a), Instruction::AddRelative { offset, amount: b }]
            | [Instruction::AddRelative { offset, amount: b }, Instruction::Add(a)] => {
                let offset = *offset;

                if offset > 0 && offset < 4 {
                    matched = true;

                    let mut vector = [0; 4];
                    vector[0] = *a;
                    vector[offset as usize] = *b;

                    buffer.push(Instruction::AddVector { vector });
                }
            }
            [Instruction::Add(amount), Instruction::AddVector { vector }] => {
                matched = true;
                buffer.push(Instruction::AddVector {
                    vector: [
                        vector[0].wrapping_add(*amount),
                        vector[1],
                        vector[2],
                        vector[3],
                    ],
                });
            }
            [Instruction::SetValue(value), Instruction::Add(amount)] => {
                matched = true;
                buffer.push(Instruction::SetValue(value.wrapping_add(*amount)));
            }
            [Instruction::SetValue(0), Instruction::MoveRightToZero { .. } | Instruction::MoveLeftToZero { .. }] =>
            {
                matched = true;
                buffer.push(Instruction::SetValue(0));
            }
            [Instruction::AddRelative { offset, amount }, Instruction::AddVector { vector }] => {
                let offset = *offset;

                if offset >= 0 && offset < 4 {
                    matched = true;

                    let mut vector = *vector;
                    vector[offset as usize] = vector[offset as usize].wrapping_add(*amount);

                    buffer.push(Instruction::AddVector { vector });
                }
            }
            [Instruction::AddVector { vector }, Instruction::Add(amount)] => {
                matched = true;

                let mut vector = *vector;
                vector[0] = vector[0].wrapping_add(*amount);

                buffer.push(Instruction::AddVector { vector });
            }
            [first @ Instruction::MoveRightToZero { .. }
            | first @ Instruction::MoveLeftToZero { .. }, Instruction::Add(amount)] => {
                matched = true;
                buffer.extend_from_slice(&[*first, Instruction::SetValue(*amount)]);
            }
            _ => {}
        }

        if matched {
            iterator.next();
        } else {
            buffer.push(window[0]);
        }
    }

    if !matched {
        buffer.push(instructions[instructions.len() - 1]);
    }

    instructions.clear();
    mem::swap(instructions, buffer);
}

fn substitute_patterns_3(instructions: &mut Vec<Instruction>, buffer: &mut Vec<Instruction>) {
    if instructions.len() < 3 {
        return;
    }

    let mut matched = false;
    let mut iterator = instructions.windows(3);

    while let Some(window) = iterator.next() {
        matched = false;

        match window {
            [Instruction::Add(a), Instruction::Move(stride), Instruction::Add(b)] => {
                let stride = *stride;

                if stride >= 0 && stride < 4 {
                    matched = true;

                    let mut vector = [0; 4];
                    vector[0] = *a;
                    vector[stride as usize] = *b;

                    buffer.extend_from_slice(&[
                        Instruction::AddVector { vector },
                        Instruction::Move(stride),
                    ]);
                } else if stride < 0 && stride > -4 {
                    matched = true;

                    let mut vector = [0; 4];
                    vector[0] = *b;
                    vector[-stride as usize] = *a;

                    buffer.extend_from_slice(&[
                        Instruction::Move(stride),
                        Instruction::AddVector { vector },
                    ]);
                }
            }
            [Instruction::Move(move1), Instruction::Add(amount), Instruction::Move(move2)] => {
                let move1 = *move1;
                let move2 = *move2;
                let amount = *amount;

                matched = true;

                if move1 == -move2 {
                    buffer.push(Instruction::AddRelative {
                        offset: move1,
                        amount,
                    });
                } else {
                    buffer.extend_from_slice(&[
                        Instruction::AddRelative {
                            offset: move1,
                            amount,
                        },
                        Instruction::Move(move1 + move2),
                    ]);
                }
            }
            [Instruction::JumpIfZero { .. }, Instruction::Add(1 | -1), Instruction::JumpIfNotZero { .. }] =>
            {
                matched = true;
                buffer.push(Instruction::SetValue(0));
            }
            [Instruction::JumpIfZero { .. }, Instruction::Move(stride), Instruction::JumpIfNotZero { .. }] =>
            {
                matched = true;
                let stride = *stride;

                if stride > 0 {
                    buffer.push(Instruction::MoveRightToZero {
                        increment: 0,
                        stride: stride as usize,
                    });
                } else if stride < 0 {
                    buffer.push(Instruction::MoveLeftToZero {
                        increment: 0,
                        stride: stride.unsigned_abs(),
                    });
                }
            }
            [Instruction::AddRelative {
                offset: offset1,
                amount: amount1,
            }, inst @ _, Instruction::AddRelative {
                offset: offset2,
                amount: amount2,
            }] => {
                if *offset1 == *offset2 && inst.preserves_tape_head() {
                    matched = true;
                    buffer.extend_from_slice(&[
                        Instruction::AddRelative {
                            offset: *offset1,
                            amount: *amount1 + *amount2,
                        },
                        *inst,
                    ]);
                }
            }
            _ => {}
        }

        if matched {
            iterator.next();
            iterator.next();
        } else {
            buffer.push(window[0]);
        }
    }

    if !matched {
        buffer.push(instructions[instructions.len() - 2]);
        buffer.push(instructions[instructions.len() - 1]);
    }

    instructions.clear();
    mem::swap(instructions, buffer);
}

fn substitute_patterns_4(instructions: &mut Vec<Instruction>, buffer: &mut Vec<Instruction>) {
    if instructions.len() < 4 {
        return;
    }

    let mut matched = false;
    let mut iterator = instructions.windows(4);

    while let Some(window) = iterator.next() {
        matched = false;

        match window {
            [Instruction::Add(a), Instruction::Move(move1), Instruction::Add(b), Instruction::Move(move2)] =>
            {
                let move1 = *move1;
                let move2 = *move2;
                let total_move = move1 + move2;

                if move1 > 0 && move2 > 0 && total_move < 4 {
                    matched = true;

                    let mut vector = [0; 4];
                    vector[0] = *a;
                    vector[move1 as usize] = *b;

                    buffer.extend_from_slice(&[
                        Instruction::AddVector { vector },
                        Instruction::Move(total_move),
                    ]);
                } else if move1 < 0 && move2 < 0 && total_move > -4 {
                    matched = true;

                    let mut vector = [0; 4];
                    vector[1] = *b;
                    vector[(-move1 as usize) + 1] = *a;

                    buffer.extend_from_slice(&[
                        Instruction::Move(total_move),
                        Instruction::AddVector { vector },
                    ]);
                }
            }
            [Instruction::Move(move1), Instruction::Add(a), Instruction::Move(move2), Instruction::Add(b)] =>
            {
                let move1 = *move1;
                let move2 = *move2;
                let total_move = move1 + move2;

                if move1 > 0 && move2 > 0 && total_move < 4 {
                    matched = true;

                    let mut vector = [0; 4];
                    vector[move1 as usize] = *a;
                    vector[total_move as usize] = *b;

                    buffer.extend_from_slice(&[
                        Instruction::AddVector { vector },
                        Instruction::Move(total_move),
                    ]);
                } else if move1 < 0 && move2 < 0 && total_move > -4 {
                    matched = true;

                    let mut vector = [0; 4];
                    vector[0] = *b;
                    vector[-move2 as usize] = *a;

                    buffer.extend_from_slice(&[
                        Instruction::Move(total_move),
                        Instruction::AddVector { vector },
                    ]);
                }
            }
            [Instruction::JumpIfZero { .. }, Instruction::Add(increment), Instruction::Move(stride), Instruction::JumpIfNotZero { .. }] =>
            {
                matched = true;

                if *stride > 0 {
                    buffer.push(Instruction::MoveRightToZero {
                        increment: *increment,
                        stride: *stride as usize,
                    });
                } else if *stride < 0 {
                    buffer.push(Instruction::MoveLeftToZero {
                        increment: *increment,
                        stride: stride.unsigned_abs(),
                    });
                }
            }
            [Instruction::AddRelative {
                offset: offset1,
                amount: amount1,
            }, inst1 @ _, inst2 @ _, Instruction::AddRelative {
                offset: offset2,
                amount: amount2,
            }] => {
                if *offset1 == *offset2 && inst1.is_add_friendly() && inst2.is_add_friendly() {
                    matched = true;
                    buffer.extend_from_slice(&[
                        Instruction::AddRelative {
                            offset: *offset1,
                            amount: *amount1 + *amount2,
                        },
                        *inst1,
                        *inst2,
                    ]);
                }
            }
            _ => {
                matched = false;
            }
        }

        if matched {
            iterator.next();
            iterator.next();
            iterator.next();
        } else {
            buffer.push(window[0]);
        }
    }

    if !matched {
        buffer.push(instructions[instructions.len() - 3]);
        buffer.push(instructions[instructions.len() - 2]);
        buffer.push(instructions[instructions.len() - 1]);
    }

    instructions.clear();
    mem::swap(instructions, buffer);
}

fn remove_spurious_loops(instructions: &mut Vec<Instruction>, buffer: &mut Vec<Instruction>) {
    {
        let mut cell_is_zero = true;
        let mut iterator = instructions.drain(..);

        'loop_squash: while let Some(instruction) = iterator.next() {
            match instruction {
                Instruction::JumpIfZero { .. } => {
                    if cell_is_zero {
                        let mut loop_depth = 0;

                        while let Some(next_instruction) = iterator.next() {
                            match next_instruction {
                                Instruction::JumpIfZero { .. } => {
                                    loop_depth += 1;
                                }
                                Instruction::JumpIfNotZero { .. } => {
                                    if loop_depth == 0 {
                                        continue 'loop_squash;
                                    } else {
                                        loop_depth -= 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Instruction::Write(_) => {}
                Instruction::JumpIfNotZero { .. }
                | Instruction::MoveRightToZero { .. }
                | Instruction::MoveLeftToZero { .. } => {
                    cell_is_zero = true;
                }
                Instruction::Add(_)
                | Instruction::Move(_)
                | Instruction::Read(_)
                | Instruction::AddRelative { .. }
                | Instruction::AddVector { .. } => {
                    cell_is_zero = false;
                }
                Instruction::SetValue(value) => {
                    cell_is_zero = value == 0;
                }
            }

            buffer.push(instruction)
        }
    }

    mem::swap(instructions, buffer);
}

fn fix_loops(instructions: &mut Vec<Instruction>) {
    let mut jump_stack = Vec::new();

    for (index, instruction) in instructions.iter_mut().enumerate() {
        match instruction {
            Instruction::JumpIfZero { location } => {
                *location = index;
                jump_stack.push(instruction);
            }
            Instruction::JumpIfNotZero {
                location: loop_start,
            } => {
                if let Some(Instruction::JumpIfZero { location: loop_end }) = jump_stack.pop() {
                    *loop_start = *loop_end;
                    *loop_end = index;
                } else {
                    // TODO: ICE
                }
            }
            _ => {}
        }
    }
}
