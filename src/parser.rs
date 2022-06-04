/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::instruction::Instruction;

pub fn parse_file(filename: &str) -> Result<Vec<Instruction>, String> {
    let file = File::open(filename).map_err(|err| err.to_string())?;

    let mut instructions = Vec::new();
    let mut jump_stack = Vec::new();

    for line in BufReader::new(file).lines() {
        let line = line.map_err(|err| err.to_string())?;
        parse(&mut instructions, &mut jump_stack, &line)?;
    }

    Ok(instructions)
}

pub fn parse_string(string: &str) -> Result<Vec<Instruction>, String> {
    let mut instructions = Vec::new();
    let mut jump_stack = Vec::new();
    parse(&mut instructions, &mut jump_stack, string)?;
    Ok(instructions)
}

fn parse(
    instructions: &mut Vec<Instruction>,
    jump_stack: &mut Vec<usize>,
    string: &str,
) -> Result<(), String> {
    for c in string.chars() {
        match c {
            '+' => instructions.push(Instruction::Add(1)),
            '-' => instructions.push(Instruction::Add(-1)),
            '>' => instructions.push(Instruction::Move(1)),
            '<' => instructions.push(Instruction::Move(-1)),
            '.' => instructions.push(Instruction::Write(1)),
            ',' => instructions.push(Instruction::Read(1)),
            '[' => {
                jump_stack.push(instructions.len());
                instructions.push(Instruction::JumpIfZero { location: 0 });
            }
            ']' => {
                if let Some(loop_start) = jump_stack.pop() {
                    let instruction_count = instructions.len();

                    if let Some(Instruction::JumpIfZero { location: loop_end }) =
                        instructions.get_mut(loop_start)
                    {
                        *loop_end = instruction_count;
                        instructions.push(Instruction::JumpIfNotZero {
                            location: loop_start,
                        });
                    } else {
                        // TODO: Throw a proper error here; ice.
                        return Err("ERROR".to_owned());
                    }
                } else {
                    // TODO: Throw a proper error here.
                    return Err("ERROR".to_owned());
                }
            }
            _ => {}
        }
    }

    Ok(())
}
