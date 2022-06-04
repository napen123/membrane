/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom};
use std::time::Instant;

use clap::Parser;

use interpreter::{InputSource, OutputSource, TapeSize};

mod compiler;
mod instruction;
mod interpreter;
mod lister;
mod optimizer;
mod parser;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Args {
    #[clap(
        short,
        long,
        help = "Print additional information during program execution."
    )]
    verbose: bool,

    #[clap(
        short,
        long,
        help = "Perform a partial execution by _not_ running the interpreter."
    )]
    partial: bool,

    #[clap(
        short,
        long,
        help = "Perform optimizations before interpreting, listing, and compiling."
    )]
    optimize: bool,

    #[clap(
        short = 'R',
        long,
        help = "Buffer reads instead of performing them on-demand. This can improve the performance of programs with frequent reads, potentially at the cost of user interactivity."
    )]
    buffer_read: bool,

    #[clap(
        short = 'W',
        long,
        help = "Buffer writes instead of flushing them immediately. This can improve the performance of programs with frequent writes, potentially at the cost of user interactivity."
    )]
    buffer_write: bool,

    #[clap(
        short,
        long = "tape",
        help = "The tape size to use while optimizing, interpreting, and compiling. Zero (0) corresponds to a right-infinite tape, and positive values correspond to a finite, wrapping tape.",
        default_value_t = 0
    )]
    tape_size: usize,

    #[clap(
        short,
        long = "read",
        help = "Used to specify a file to use for program input in place of stdin. If program input is set to buffered, then the specified file will be buffered/streamed; otherwise, the entire contents of the specified file will be read into memory."
    )]
    read_file: Option<String>,

    #[clap(
        short,
        long = "write",
        help = "Used to specify a file to use for program output in place of stdout. If program output is set to buffered, then the program's output will be buffered before being flushed into the specified file; otherwise, all writes will result in a flush."
    )]
    write_file: Option<String>,

    #[clap(
        short,
        long = "listing",
        help = "An optional listing file to fill with instructions and membrane data. (Created after optimizations.)"
    )]
    listing_file: Option<String>,

    #[clap(
        short,
        long = "compile",
        help = "An optional C file to compile the input code to. (Created after optimizations.)"
    )]
    c_file: Option<String>,

    #[clap(help = "The Brainfuck file to interpret or compile.")]
    brainfuck_file: String,
}

fn main() {
    let args = Args::parse();

    let mut instructions = parser::parse_file(&args.brainfuck_file).unwrap();

    if args.optimize {
        optimizer::optimize(args.verbose, &mut instructions);
    }

    if let Some(listing_file) = args.listing_file {
        lister::create_listing(&instructions, listing_file).unwrap();
    }

    if let Some(c_file) = args.c_file {
        compiler::compile_to_c(&instructions, c_file).unwrap();
    }

    if !args.partial {
        // TODO: Customizable input source with read_file.
        // TODO: Customizable output source with write_file.
        // TODO: Take the buffer flag into account.

        let tape_size = if args.tape_size == 0 {
            TapeSize::Infinite
        } else {
            TapeSize::Finite(args.tape_size)
        };

        let input = if let Some(filename) = args.read_file {
            let mut file = File::open(filename).unwrap();

            if args.buffer_read {
                InputSource::FileBuffer(BufReader::new(file))
            } else {
                let mut contents = match file.seek(SeekFrom::End(0)) {
                    Ok(end) => match file.seek(SeekFrom::Start(0)) {
                        Ok(start) => Vec::with_capacity((start - end) as usize),
                        Err(_) => Vec::new(),
                    },
                    Err(_) => Vec::new(),
                };

                match file.read_to_end(&mut contents) {
                    Ok(_) => InputSource::File(Cursor::new(contents)),
                    Err(_) => {
                        // TODO: Throw an error here; failed to read contents of file.
                        return;
                    }
                }
            }
        } else if args.buffer_read {
            InputSource::StdinBuffer(BufReader::new(io::stdin()))
        } else {
            InputSource::Stdin(io::stdin())
        };

        let output = if let Some(filename) = args.write_file {
            let file = File::create(filename).unwrap();

            if args.buffer_write {
                OutputSource::FileBuffer(BufWriter::new(file))
            } else {
                OutputSource::File(file)
            }
        } else if args.buffer_write {
            OutputSource::StdoutBuffer(BufWriter::new(io::stdout()))
        } else {
            OutputSource::Stdout(io::stdout())
        };

        let start_time = args.verbose.then(Instant::now);
        interpreter::interpret(&instructions, input, output, tape_size);

        if let Some(time) = start_time {
            let elapsed_ms = time.elapsed().as_millis();
            println!("Execution took {} ms.", elapsed_ms);
        }
    }
}
