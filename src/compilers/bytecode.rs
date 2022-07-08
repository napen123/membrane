use std::io::{Result as IOResult, Write};

use crate::instruction::Instruction;
use crate::interpreter::TapeSize;

const BYTECODE_VERSION: u8 = 1;

const OPCODE_ADD: u8 = 1;
const OPCODE_MOVE: u8 = 2;

pub fn compile_to_bytecode<W: Write>(
    instructions: &[Instruction],
    tape_size: TapeSize,
    writer: &mut W,
) -> IOResult<()> {
    writer.write_all(&[b'B', b'F', b'C', BYTECODE_VERSION])?;

    for instruction in instructions {
        match instruction {
            Instruction::Add(amount) => {
                writer.write_all(&[OPCODE_ADD, *amount as u8])?;
            }
            Instruction::Move(amount) => {
                writer.write_all(&[OPCODE_MOVE, *amount as u8])?;
            }
            _ => {}
        }
    }

    Ok(())
}
