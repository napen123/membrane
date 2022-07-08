/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Stdin, Stdout, Write};
use std::iter;

use crate::instruction::Instruction;

const VECTOR_SIZE: usize = 4;
const TAPE_GROW_AMOUNT: usize = 50;
const STANDARD_TAPE_SIZE: usize = 30_000;
const DEFAULT_INPUT_BUFFER_SIZE: usize = 8;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TapeSize {
    Finite(usize),
    Infinite,
}

pub enum InputSource {
    Stdin(Stdin),
    StdinBuffer(BufReader<Stdin>),
    File(Cursor<Vec<u8>>),
    FileBuffer(BufReader<File>),
}

impl Read for InputSource {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Stdin(stdin) => stdin.read(buf),
            Self::StdinBuffer(reader) => reader.read(buf),
            Self::File(cursor) => cursor.read(buf),
            Self::FileBuffer(reader) => reader.read(buf),
        }
    }
}

pub enum OutputSource {
    Stdout(Stdout),
    StdoutBuffer(BufWriter<Stdout>),
    File(File),
    FileBuffer(BufWriter<File>),
}

impl Write for OutputSource {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Stdout(stdout) => stdout.write(buf),
            Self::StdoutBuffer(writer) => writer.write(buf),
            Self::File(file) => file.write(buf),
            Self::FileBuffer(writer) => writer.write(buf),
        }
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Stdout(stdout) => stdout.flush(),
            Self::StdoutBuffer(writer) => writer.flush(),
            Self::File(file) => file.flush(),
            Self::FileBuffer(writer) => writer.flush(),
        }
    }
}

struct Memory {
    head: usize,
    tape: Vec<u8>,
    size: TapeSize,
}

impl Memory {
    fn new(size: TapeSize) -> Self {
        let length = if let TapeSize::Finite(tape_size) = size {
            tape_size
        } else {
            STANDARD_TAPE_SIZE
        };

        Self {
            head: 0,
            tape: vec![0; length],
            size,
        }
    }

    fn move_head(&mut self, amount: isize) {
        if amount >= 0 {
            self.move_head_right(amount as usize)
        } else {
            self.move_head_left(-amount as usize)
        }
    }

    fn move_head_right(&mut self, amount: usize) {
        match self.size {
            TapeSize::Finite(tape_size) => {
                self.head = self.head.wrapping_add(amount).wrapping_rem(tape_size);
            }
            TapeSize::Infinite => {
                self.head = self.head.saturating_add(amount);
            }
        }
    }

    fn move_head_left(&mut self, amount: usize) {
        match self.size {
            TapeSize::Finite(tape_size) => {
                self.head = self.head.wrapping_sub(amount).wrapping_rem(tape_size);
            }
            TapeSize::Infinite => {
                self.head = self.head.saturating_sub(amount);
            }
        }
    }

    #[inline]
    fn current_cell_value(&self) -> u8 {
        self.get_cell_value(self.head)
    }

    #[inline]
    fn current_cell_mut(&mut self) -> &mut u8 {
        self.get_cell_mut(self.head)
    }

    fn current_cell_vector(&mut self) -> [usize; VECTOR_SIZE] {
        match self.size {
            TapeSize::Finite(tape_size) => {
                let head0 = self.head;
                let head1 = self.head.wrapping_add(1).wrapping_rem(tape_size);
                let head2 = self.head.wrapping_add(2).wrapping_rem(tape_size);
                let head3 = self.head.wrapping_add(3).wrapping_rem(tape_size);
                [head0, head1, head2, head3]
            }
            TapeSize::Infinite => {
                if self.head + VECTOR_SIZE >= self.tape.len() {
                    self.tape.extend(iter::repeat(0).take(TAPE_GROW_AMOUNT));
                }

                let head0 = self.head;
                let head1 = self.head.saturating_add(1);
                let head2 = self.head.saturating_add(2);
                let head3 = self.head.saturating_add(3);
                [head0, head1, head2, head3]
            }
        }
    }

    fn get_cell_value(&self, index: usize) -> u8 {
        match self.size {
            TapeSize::Finite(tape_size) => {
                let wrapped_index = index.wrapping_rem(tape_size);

                // SAFETY: index is modded against tape_size,
                // which should never exceed the tape's length.
                unsafe { *self.tape.get_unchecked(wrapped_index) }
            }
            TapeSize::Infinite => self.tape.get(index).copied().unwrap_or_default(),
        }
    }

    fn get_cell_mut(&mut self, index: usize) -> &mut u8 {
        match self.size {
            TapeSize::Finite(tape_size) => {
                let wrapped_index = index.wrapping_rem(tape_size);

                // SAFETY: index is modded against tape_size,
                // which should never exceed the tape's length.
                unsafe { self.tape.get_unchecked_mut(wrapped_index) }
            }
            TapeSize::Infinite => {
                let tape_size = self.tape.len();

                if index >= tape_size {
                    let amount_to_grow = index.saturating_add(TAPE_GROW_AMOUNT) - tape_size;
                    self.tape.extend(iter::repeat(0).take(amount_to_grow));
                }

                // SAFETY: The above check ensures index is in-bounds.
                unsafe { self.tape.get_unchecked_mut(index) }
            }
        }
    }
}

pub fn interpret(
    instructions: &[Instruction],
    mut input: InputSource,
    mut output: OutputSource,
    tape_size: TapeSize,
) -> usize {
    let mut program_counter = 0;
    let mut memory = Memory::new(tape_size);

    let mut io_buffer = vec![0u8; DEFAULT_INPUT_BUFFER_SIZE];

    let mut instructions_executed = 0;

    while let Some(instruction) = instructions.get(program_counter) {
        program_counter += 1;
        instructions_executed += 1;

        match instruction {
            Instruction::Add(amount) => {
                let cell = memory.current_cell_mut();

                // TODO: Use std's u8.wrapping_add_signed once its stabilized.
                *cell = cell.wrapping_add(*amount as u8);
            }
            Instruction::Move(amount) => memory.move_head(*amount),
            Instruction::Write(amount) => {
                let amount = *amount;
                let cell = memory.current_cell_value();

                if amount >= io_buffer.len() {
                    let amount_to_grow = amount + 1 - io_buffer.len();
                    io_buffer.extend(iter::repeat(0).take(amount_to_grow));
                }

                let slice = &mut io_buffer[0..amount];
                slice.fill(cell);

                let _lock = if let OutputSource::Stdout(ref stdout) = output {
                    Some(stdout.lock())
                } else {
                    None
                };

                match output.write_all(slice) {
                    Ok(_) => {}
                    Err(_) => {
                        // TODO: Throw an error here; failed to write all output.
                        todo!()
                    }
                }
            }
            Instruction::Read(amount) => {
                let amount = *amount;

                if amount > 0 {
                    if amount >= io_buffer.len() {
                        let amount_to_grow = amount + 1 - io_buffer.len();
                        io_buffer.extend(iter::repeat(0).take(amount_to_grow));
                    }

                    match input.read_exact(&mut io_buffer[0..amount]) {
                        Ok(_) => {
                            let cell = memory.current_cell_mut();

                            // SAFETY: Since amount > 0, there must be a last element.
                            *cell = unsafe { *io_buffer.last().unwrap_unchecked() };
                        }
                        Err(_) => {
                            // TODO: Throw an error here; reading from input source failed.
                            todo!()
                        }
                    }
                }
            }
            Instruction::JumpIfZero { location } => {
                let cell = memory.current_cell_value();

                if cell == 0 {
                    program_counter = *location;
                }
            }
            Instruction::JumpIfNotZero { location } => {
                let cell = memory.current_cell_value();

                if cell != 0 {
                    program_counter = *location;
                }
            }

            Instruction::SetValue(value) => {
                let cell = memory.current_cell_mut();
                *cell = *value as u8;
            }
            Instruction::AddRelative { offset, amount } => {
                let offset = *offset;

                let index = match tape_size {
                    TapeSize::Finite(_) => {
                        // TODO: Use std's usize.wrapping_add_signed once its stabilized.
                        memory.head.wrapping_add(offset as usize)
                    }
                    TapeSize::Infinite => {
                        if offset >= 0 {
                            // TODO: Use std's usize.saturating_add_signed once its stabilized.
                            memory.head.saturating_add(offset as usize)
                        } else {
                            // TODO: Use std's usize.saturating_sub_signed once its stabilized.
                            memory.head.saturating_sub(-offset as usize)
                        }
                    }
                };

                let cell = memory.get_cell_mut(index);

                // TODO: Use std's u8.wrapping_add_signed once its stabilized.
                *cell = cell.wrapping_add(*amount as u8);
            }
            Instruction::AddVector { vector: amount } => {
                let vector = memory.current_cell_vector();

                // SAFETY: current_cell_vector() ensures the returned indices are in-bounds.
                unsafe {
                    for i in 0..VECTOR_SIZE {
                        let cell = memory.tape.get_unchecked_mut(vector[i]);

                        // TODO: Use std's u8.wrapping_add_signed once its stabilized.
                        *cell = cell.wrapping_add(amount[i] as u8);
                    }
                }
            }
            Instruction::MoveRightToZero { increment, stride } => {
                let mut cell = memory.current_cell_mut();

                while *cell != 0 {
                    // TODO: Use std's u8.wrapping_add_signed once its stabilized.
                    *cell = cell.wrapping_add(*increment as u8);
                    memory.move_head_right(*stride);
                    cell = memory.current_cell_mut();
                }
            }
            Instruction::MoveLeftToZero { increment, stride } => {
                let mut cell = memory.current_cell_mut();

                while *cell != 0 {
                    // TODO: Use std's u8.wrapping_add_signed once its stabilized.
                    *cell = cell.wrapping_add(*increment as u8);
                    memory.move_head_left(*stride);
                    cell = memory.current_cell_mut();
                }
            }
        }
    }

    match output.flush() {
        Ok(_) => instructions_executed,
        Err(_) => {
            // TODO: Throw an error here; we failed to flush output!
            todo!()
        }
    }
}
