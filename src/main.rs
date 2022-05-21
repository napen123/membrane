/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use instruction::Instruction;

mod compiler;
mod instruction;
mod interpreter;
mod lister;
mod optimizer;
mod parser;

fn main() {
    let mut instructions = parser::parse_file("examples/numwarp.bf").unwrap();
    let input = [
        '2' as u8, '5' as u8, '9' as u8, '7' as u8, // ' ' as u8, '\n' as u8,
    ];

    optimizer::optimize(&mut instructions);

    lister::create_listing(&instructions, "numwarp.lst").unwrap();
    //compiler::compile_to_c(&instructions, "numwarp.c").unwrap();

    let start_time = std::time::Instant::now();
    interpreter::interpret(&instructions, &input);
    println!("{} ms", start_time.elapsed().as_millis());
}
