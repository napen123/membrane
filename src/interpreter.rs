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

    fn move_head(&mut self, amount: isize) -> Result<(), ()> {
        match self.size {
            TapeSize::Finite(tape_size) => {
                self.head = ((self.head as isize + amount) % tape_size as isize) as usize;
                Ok(())
            }
            TapeSize::Infinite => {
                let new_head = self.head as isize + amount;

                if new_head >= 0 {
                    self.head = new_head as usize;
                    Ok(())
                } else {
                    Err(())
                }
            }
        }
    }

    fn move_head_right(&mut self, amount: usize) {
        match self.size {
            TapeSize::Finite(tape_size) => {
                self.head = (self.head + amount) % tape_size;
            }
            TapeSize::Infinite => {
                self.head += amount;
            }
        }
    }

    fn move_head_left(&mut self, amount: usize) -> Result<(), ()> {
        match self.size {
            TapeSize::Finite(tape_size) => {
                self.head = ((self.head as isize + amount as isize) % tape_size as isize) as usize;
                Ok(())
            }
            TapeSize::Infinite => {
                if amount <= self.head {
                    self.head -= amount;
                    Ok(())
                } else {
                    Err(())
                }
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
                let head1 = (self.head + 1) % tape_size;
                let head2 = (self.head + 2) % tape_size;
                let head3 = (self.head + 3) % tape_size;
                [head0, head1, head2, head3]
            }
            TapeSize::Infinite => {
                if self.head + VECTOR_SIZE >= self.tape.len() {
                    self.tape.extend(iter::repeat(0).take(VECTOR_SIZE));
                }

                let head0 = self.head;
                let head1 = self.head + 1;
                let head2 = self.head + 2;
                let head3 = self.head + 3;
                [head0, head1, head2, head3]
            }
        }
    }

    fn get_cell_value(&self, index: usize) -> u8 {
        match self.size {
            TapeSize::Finite(tape_size) => {
                let wrapped_index = index % tape_size;
                unsafe { *self.tape.get_unchecked(wrapped_index) }
            }
            TapeSize::Infinite => self.tape.get(index).copied().unwrap_or_default(),
        }
    }

    fn get_cell_mut(&mut self, index: usize) -> &mut u8 {
        match self.size {
            TapeSize::Finite(tape_size) => {
                let wrapped_index = index % tape_size;
                unsafe { self.tape.get_unchecked_mut(wrapped_index) }
            }
            TapeSize::Infinite => {
                let tape_size = self.tape.len();

                if index >= tape_size {
                    self.tape
                        .extend(iter::repeat(0).take(index + 1 - tape_size));
                }

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
) {
    let mut program_counter = 0;
    let mut memory = Memory::new(tape_size);

    let mut io_buffer = vec![0u8; DEFAULT_INPUT_BUFFER_SIZE];

    while let Some(instruction) = instructions.get(program_counter) {
        program_counter += 1;

        match instruction {
            Instruction::Add(amount) => {
                let cell = memory.current_cell_mut();
                *cell = (*cell as i8).wrapping_add(*amount) as u8;
            }
            Instruction::Move(amount) => match memory.move_head(*amount) {
                Ok(_) => {}
                Err(_) => {
                    // TODO: Throw an error here; the tape was moved out of bounds.
                    return;
                }
            },
            Instruction::Write(amount) => {
                let amount = *amount;
                let cell = memory.current_cell_value();

                if amount >= io_buffer.len() {
                    io_buffer.extend(iter::repeat(0).take(amount + 1 - io_buffer.len()));
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
                        return;
                    }
                }
            }
            Instruction::Read(amount) => {
                let amount = *amount;

                if amount >= io_buffer.len() {
                    io_buffer.extend(iter::repeat(0).take(amount + 1 - io_buffer.len()));
                }

                match input.read_exact(&mut io_buffer[0..amount]) {
                    Ok(_) => {
                        let cell = memory.current_cell_mut();
                        *cell = io_buffer[amount - 1];
                    }
                    Err(_) => {
                        // TODO: Throw an error here; reading from input source failed.
                        return;
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
                let head = memory.head as isize;
                let index = head + *offset;

                if index >= 0 {
                    let cell = memory.get_cell_mut(index as usize);
                    *cell = (*cell as i8).wrapping_add(*amount) as u8;
                } else {
                    // TODO: Throw an error here; tried to add to a negative index.
                    return;
                }
            }
            Instruction::AddVector { vector: amount } => {
                let vector = memory.current_cell_vector();

                // TODO: SAFETY
                unsafe {
                    for i in 0..VECTOR_SIZE {
                        let cell = memory.tape.get_unchecked_mut(vector[i]);
                        *cell = (*cell as i8).wrapping_add(amount[i]) as u8;
                    }
                }
            }
            Instruction::MoveRightToZero { increment, stride } => {
                let mut cell = memory.current_cell_mut();

                while *cell != 0 {
                    *cell = (*cell as i8).wrapping_add(*increment) as u8;
                    memory.move_head_right(*stride);
                    cell = memory.current_cell_mut();
                }
            }
            Instruction::MoveLeftToZero { increment, stride } => {
                let mut cell = memory.current_cell_mut();

                while *cell != 0 {
                    *cell = (*cell as i8).wrapping_add(*increment) as u8;

                    match memory.move_head_left(*stride) {
                        Ok(_) => {
                            cell = memory.current_cell_mut();
                        }
                        Err(_) => {
                            // TODO: Throw an error here; the tape was moved out of bounds.
                            return;
                        }
                    }
                }
            }
        }
    }

    match output.flush() {
        Ok(_) => {}
        Err(_) => {
            // TODO: Throw an error here; we failed to flush output!
        }
    }
}
