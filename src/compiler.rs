/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fs::File;
use std::io::{BufWriter, Result as IOResult, Write};

use crate::Instruction;

// TODO: Output is broken -- segfault.
// This is only an example at the moment; a rewrite is in order.
pub fn compile_to_c(instructions: &[Instruction], filename: &str) -> IOResult<()> {
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "#include <stdio.h>")?;
    writeln!(writer)?;

    writeln!(writer, "int head = 0;")?;
    writeln!(writer, "unsigned char tape[30000] = {{0}};")?;
    writeln!(writer)?;

    writeln!(writer, "int main(int argc, const char **argv) {{")?;

    for (index, instruction) in instructions.iter().enumerate() {
        match instruction {
            Instruction::Add(amount) => {
                writeln!(writer, "    tape[head] += {};", amount)?;
            }
            Instruction::Move(amount) => {
                if *amount >= 0 {
                    writeln!(writer, "    head += {};", *amount)?;
                } else {
                    writeln!(writer, "    head -= {};", *amount)?;
                }
            }
            Instruction::Write(amount) => {
                for _ in 1..*amount {
                    writeln!(writer, "    printf(\"%c\", tape[head]);")?;
                }
            }
            Instruction::Read(amount) => {
                for _ in 1..*amount {
                    writeln!(writer, "    tape[head] = getchar();")?;
                }
            }
            Instruction::JumpIfZero { location } => {
                writeln!(writer, "jump_{}:", index)?;
                writeln!(writer, "    if (tape[head] == 0) {{")?;
                writeln!(writer, "        goto jump_{};", location)?;
                writeln!(writer, "    }}")?;
            }
            Instruction::JumpIfNotZero { location } => {
                writeln!(writer, "jump_{}:", index)?;
                writeln!(writer, "    if (tape[head] != 0) {{")?;
                writeln!(writer, "        goto jump_{};", location)?;
                writeln!(writer, "    }}")?;
            }

            Instruction::SetAbsolute(value) => {
                writeln!(writer, "    tape[head] = {};", value)?;
            }
            Instruction::AddRelative { offset, amount } => {
                writeln!(writer, "    tape[head + {}] += {};", offset, amount)?;
            }
            Instruction::AddVector { stride, vector } => {
                writeln!(writer, "    tape[head] += {};", vector[0])?;
                writeln!(writer, "    tape[head + 1] += {};", vector[1])?;
                writeln!(writer, "    tape[head + 2] += {};", vector[2])?;
                writeln!(writer, "    tape[head + 3] += {};", vector[3])?;
                writeln!(writer, "    head += {};", *stride)?;
            }
            Instruction::MoveRightToZero { increment, stride } => {
                writeln!(writer, "    while (tape[head] != 0) {{")?;

                if *increment != 0 {
                    writeln!(writer, "        tape[head] += {};", increment)?;
                }

                writeln!(writer, "        head += {};", stride)?;
                writeln!(writer, "    }}")?;
            }
            Instruction::MoveLeftToZero { increment, stride } => {
                writeln!(writer, "    while (tape[head] != 0) {{")?;

                if *increment != 0 {
                    writeln!(writer, "        tape[head] += {};", increment)?;
                }

                writeln!(writer, "        head -= {};", stride)?;
                writeln!(writer, "    }}")?;
            }
        }
    }

    writeln!(writer, "    return 0;")?;
    writeln!(writer, "}}")?;

    Ok(())
}
