use std::fs::File;
use std::io::{BufWriter, Result as IOResult, Write};
use std::path::Path;

use clap::ArgEnum;

use crate::instruction::Instruction;
use crate::interpreter::TapeSize;

mod bytecode;
mod rust;

#[derive(Copy, Clone, Eq, PartialEq, Default, ArgEnum)]
pub enum CompileFormat {
    #[default]
    Bytecode,
    Rust,
}

impl CompileFormat {
    pub fn compile<P: AsRef<Path>>(
        self,
        instructions: &[Instruction],
        tape_size: TapeSize,
        output_file: P,
    ) -> IOResult<()> {
        let file = File::create(output_file)?;
        let mut writer = BufWriter::new(file);

        match self {
            Self::Bytecode => bytecode::compile_to_bytecode(instructions, tape_size, &mut writer)?,
            Self::Rust => rust::compile_to_rust(instructions, tape_size, &mut writer)?,
        }

        writer.flush()
    }
}
