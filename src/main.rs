/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom};
use std::time::Instant;

use clap::{ArgAction, Parser, Subcommand};

use membrane::compilers::CompileFormat;
use membrane::interpreter::{InputSource, OutputSource, TapeSize};
use membrane::*;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Args {
    #[clap(
    short = 'v',
    long = "verbose",
    action = ArgAction::Count,
    help = "Print additional information during program execution."
    )]
    verbose: u8,

    #[clap(
        short = 'O',
        long = "optimize",
        help = "Perform optimizations before interpreting, listing, or compiling."
    )]
    optimize: bool,

    #[clap(
        short = 't',
        long = "tape",
        help = "The tape size to use while optimizing, interpreting, or compiling. Zero (0) corresponds to a right-infinite, saturating tape, and positive values correspond to a finite, wrapping tape.",
        default_value_t = 0
    )]
    tape_size: usize,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[clap(about = "Execute a Brainfuck file.")]
    Run {
        #[clap(
            short = 'R',
            long = "bufread",
            help = "Buffer reads instead of performing them on-demand. This can improve the performance of programs with frequent reads, potentially at the cost of user interactivity."
        )]
        buffer_read: bool,

        #[clap(
            short = 'W',
            long = "bufwrite",
            help = "Buffer writes instead of flushing them immediately. This can improve the performance of programs with frequent writes, potentially at the cost of user interactivity."
        )]
        buffer_write: bool,

        #[clap(
            short = 'r',
            long = "read",
            help = "Used to specify a file to use for reading input instead of using stdin. If program input is set to buffered, then the specified file will be buffered; otherwise, the entire contents of the file will be read into memory."
        )]
        read_file: Option<String>,

        #[clap(
            short = 'w',
            long = "write",
            help = "Used to specify a file to use for writing output instead of using stdout. If program output is set to buffered, then the specified file will be buffered; otherwise, all writes may result in a flush."
        )]
        write_file: Option<String>,

        #[clap(help = "The Brainfuck file to interpret.")]
        input_file: String,
    },
    #[clap(about = "Create a listing file of Brainfuck instructions.")]
    List {
        #[clap(help = "The Brainfuck file to list.")]
        input_file: String,

        #[clap(help = "The file to write the listed contents to.")]
        output_file: String,
    },
    #[clap(about = "Compile a Brainfuck file into another format.")]
    Compile {
        #[clap(
            short = 'f',
            long = "format",
            arg_enum,
            value_parser,
            default_value_t = CompileFormat::default(),
            help = "The format to compile to."
        )]
        format: CompileFormat,

        #[clap(help = "The Brainfuck file to compile.")]
        input_file: String,

        #[clap(help = "The file to write the compiled contents to.")]
        output_file: String,
    },
}

fn main() {
    let args = Args::parse();

    let mut instructions = {
        let input_file = match args.command {
            Command::Run { ref input_file, .. } => input_file,
            Command::List { ref input_file, .. } => input_file,
            Command::Compile { ref input_file, .. } => input_file,
        };

        parser::parse_file(input_file).unwrap()
    };

    let tape_size = if args.tape_size == 0 {
        TapeSize::Infinite
    } else {
        TapeSize::Finite(args.tape_size)
    };

    if args.optimize {
        optimizer::optimize(args.verbose > 1, &mut instructions, tape_size);
    }

    match args.command {
        Command::Run {
            buffer_read,
            buffer_write,
            read_file,
            write_file,
            ..
        } => {
            let input = if let Some(filename) = read_file {
                let mut file = File::open(filename).unwrap();

                if buffer_read {
                    InputSource::FileBuffer(BufReader::new(file))
                } else {
                    let mut contents = match file.seek(SeekFrom::End(0)) {
                        Ok(end) => match file.seek(SeekFrom::Start(0)) {
                            Ok(start) => Vec::with_capacity((end - start) as usize),
                            Err(_) => Vec::new(),
                        },
                        Err(_) => Vec::new(),
                    };

                    match file.read_to_end(&mut contents) {
                        Ok(_) => InputSource::File(Cursor::new(contents)),
                        Err(err) => {
                            // TODO: Throw a proper error here; failed to read contents of file.
                            panic!(
                                "Failed to read entire contents of input source file: {}",
                                err
                            );
                        }
                    }
                }
            } else if buffer_read {
                InputSource::StdinBuffer(BufReader::new(io::stdin()))
            } else {
                InputSource::Stdin(io::stdin())
            };

            let output = if let Some(filename) = write_file {
                let file = File::create(filename).unwrap();

                if buffer_write {
                    OutputSource::FileBuffer(BufWriter::new(file))
                } else {
                    OutputSource::File(file)
                }
            } else if buffer_write {
                OutputSource::StdoutBuffer(BufWriter::new(io::stdout()))
            } else {
                OutputSource::Stdout(io::stdout())
            };

            let start_time = (args.verbose > 0).then(Instant::now);
            let instructions_executed =
                interpreter::interpret(&instructions, input, output, tape_size);

            if args.verbose > 0 {
                if let Some(time) = start_time {
                    let elapsed = time.elapsed();
                    let elapsed_ms = elapsed.as_millis();
                    let inst_per_sec = (instructions_executed as f64) / elapsed.as_secs_f64();
                    println!(
                        "Execution took {} ms ({:} inst/sec).",
                        elapsed_ms, inst_per_sec as usize,
                    );
                }
            }
        }
        Command::List { output_file, .. } => {
            lister::create_listing(&instructions, output_file).unwrap();
        }
        Command::Compile {
            format,
            output_file,
            ..
        } => {
            format
                .compile(&instructions, tape_size, output_file)
                .unwrap();
        }
    }
}
