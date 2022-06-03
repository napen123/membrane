/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

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
        long = "stack",
        help = "The stack size to use while optimizing, interpreting, and compiling. Use zero (0) for a right-infinite stack."
    )]
    stack_size: usize,

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
        // TODO: stack size with stack_size.
        // TODO: input source with read_file.
        // TODO: output source with write_file.

        if args.verbose {
            let start_time = std::time::Instant::now();
            interpreter::interpret(&instructions, &[]);
            println!("Execution took {} ms.", start_time.elapsed().as_millis());
        } else {
            interpreter::interpret(&instructions, &[]);
        }
    }
}
