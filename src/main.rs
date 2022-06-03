/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::time::Instant;

use clap::Parser;

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
        short,
        long,
        help = "Buffer the interpreter's output instead of flushing immediately. This can improve the performance of programs with frequent writes, potentially at the cost of user interactivity."
    )]
    buffered: bool,

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
        help = "Used to specify a file to use for program input in place of stdin."
    )]
    read_file: Option<String>,

    #[clap(
        short,
        long = "write",
        help = "Used to specify a file to use for program output in place of stdout."
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
            interpreter::TapeSize::Infinite
        } else {
            interpreter::TapeSize::Finite(args.tape_size)
        };

        let start_time = args.verbose.then(Instant::now);

        interpreter::interpret(&instructions, &[], tape_size);

        if let Some(time) = start_time {
            let elapsed_ms = time.elapsed().as_millis();
            println!("Execution took {} ms.", elapsed_ms);
        }
    }
}
