use std::io::{Result as IOResult, Write};

use crate::instruction::Instruction;
use crate::interpreter::TapeSize;

pub fn compile_to_rust<W: Write>(
    instructions: &[Instruction],
    tape_size: TapeSize,
    writer: &mut W,
) -> IOResult<()> {
    writeln!(writer)?;
    writeln!(writer, "fn main() -> Result<(), ()> {{")?;
    writeln!(writer, "    let mut head = 0;")?;

    match tape_size {
        TapeSize::Finite(length) => {
            if length >= 256 {
                writeln!(writer, "    let mut tape = vec![0u8; {}];", length)?;
            } else {
                writeln!(writer, "    let mut tape = [0u8; {}];", length)?;
            }
        }
        TapeSize::Infinite => {
            writeln!(writer, "    let mut tape = vec![0u8; 30000];")?;
        }
    }

    writeln!(writer)?;

    let mut prefix = String::from("    ");

    for instruction in instructions.iter() {
        match instruction {
            Instruction::Add(amount) => {
                let amount = *amount;

                if amount >= 0 {
                    writeln!(
                        writer,
                        "{}tape[head] = tape[head].wrapping_add({});",
                        prefix, amount
                    )?;
                } else {
                    writeln!(
                        writer,
                        "{}tape[head] = tape[head].wrapping_sub({});",
                        prefix, -amount
                    )?;
                }
            }
            Instruction::Move(amount) => {
                let amount = *amount;

                match tape_size {
                    TapeSize::Finite(_) => {
                        if amount >= 0 {
                            writeln!(writer, "{}head = (head + {}) % tape.len();", prefix, amount)?;
                        } else {
                            writeln!(
                                writer,
                                "{}head = (head - {}) % tape.len();",
                                prefix, -amount
                            )?;
                        }
                    }
                    TapeSize::Infinite => {
                        if amount >= 0 {
                            writeln!(writer, "{}head += {};", prefix, amount)?;
                            writeln!(writer, "{}if head + 3 >= tape.len() {{", prefix)?;
                            writeln!(writer, "{}    tape.extend(std::iter::repeat(0).take(head + 5 - tape.len()));", prefix)?;
                            writeln!(writer, "{}}}", prefix)?;
                        } else {
                            writeln!(writer, "{}head -= {};", prefix, -amount)?;
                        }
                    }
                }
            }
            Instruction::Write(amount) => {
                if *amount <= 4 {
                    for _ in 0..*amount {
                        writeln!(writer, "{}print!(\"{{}}\", tape[head] as char);", prefix)?;
                    }
                } else {
                    writeln!(writer, "{}for _ in 0..{} {{", prefix, *amount)?;
                    writeln!(
                        writer,
                        "{}    print!(\"{{}}\", tape[head] as char);",
                        prefix
                    )?;
                    writeln!(writer, "{}}}", prefix)?;
                }
            }
            Instruction::Read(_) => todo!(),
            Instruction::JumpIfZero { .. } => {
                writeln!(writer, "{}while tape[head] != 0 {{", prefix)?;
                prefix.push_str("    ");
            }
            Instruction::JumpIfNotZero { .. } => {
                prefix.pop();
                prefix.pop();
                prefix.pop();
                prefix.pop();
                writeln!(writer, "{}}}", prefix)?;
            }

            Instruction::SetValue(value) => {
                writeln!(writer, "{}tape[head] = {};", prefix, *value)?;
            }

            Instruction::AddRelative { offset, amount } => {
                let offset = *offset;
                let amount = *amount;

                if amount >= 0 {
                    if offset >= 0 {
                        writeln!(
                            writer,
                            "{}tape[head + {}] = tape[head + {}].wrapping_add({});",
                            prefix, offset, offset, amount
                        )?;
                    } else {
                        writeln!(
                            writer,
                            "{}tape[head - {}] = tape[head - {}].wrapping_add({});",
                            prefix, -offset, -offset, amount
                        )?;
                    }
                } else {
                    if offset >= 0 {
                        writeln!(
                            writer,
                            "{}tape[head + {}] = tape[head + {}].wrapping_sub({});",
                            prefix, offset, offset, -amount
                        )?;
                    } else {
                        writeln!(
                            writer,
                            "{}tape[head - {}] = tape[head - {}].wrapping_sub({});",
                            prefix, -offset, -offset, -amount
                        )?;
                    }
                }
            }
            Instruction::AddVector { vector } => {
                for i in 0..4 {
                    let value = vector[i];

                    if value >= 0 {
                        writeln!(
                            writer,
                            "{}tape[head + {}] = tape[head + {}].wrapping_add({});",
                            prefix, i, i, value
                        )?;
                    } else {
                        writeln!(
                            writer,
                            "{}tape[head + {}] = tape[head + {}].wrapping_sub({});",
                            prefix, i, i, -value
                        )?;
                    }
                }
            }

            Instruction::MoveRightToZero { increment, stride } => {
                let increment = *increment;

                writeln!(writer, "{}while tape[head] != 0 {{", prefix)?;

                if increment > 0 {
                    writeln!(
                        writer,
                        "{}    tape[head] = tape[head].wrapping_add({});",
                        prefix, increment
                    )?;
                } else if increment < 0 {
                    writeln!(
                        writer,
                        "{}    tape[head] = tape[head].wrapping_sub({});",
                        prefix, -increment
                    )?;
                }

                writeln!(
                    writer,
                    "{}    head = (head + {}) % tape.len();",
                    prefix, *stride
                )?;
                writeln!(writer, "{}}}", prefix)?;
            }
            Instruction::MoveLeftToZero { increment, stride } => {
                let increment = *increment;

                writeln!(writer, "{}while tape[head] != 0 {{", prefix)?;

                if increment > 0 {
                    writeln!(
                        writer,
                        "{}    tape[head] = tape[head].wrapping_add({});",
                        prefix, increment
                    )?;
                } else if increment < 0 {
                    writeln!(
                        writer,
                        "{}    tape[head] = tape[head].wrapping_sub({});",
                        prefix, -increment
                    )?;
                }

                writeln!(
                    writer,
                    "{}    head = (head - {}) % tape.len();",
                    prefix, *stride
                )?;
                writeln!(writer, "{}}}", prefix)?;
            }
        }
    }

    writeln!(writer)?;
    writeln!(writer, "    Ok(())")?;
    writeln!(writer, "}}")?;

    Ok(())
}
