/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fs::File;
use std::io::{BufWriter, Result as IOResult, Write};

use crate::Instruction;

pub fn create_listing(instructions: &[Instruction], filename: &str) -> IOResult<()> {
    let file = File::create(filename)?;

    if instructions.len() > 0 {
        let padding = log10(instructions.len()) + 1;
        let mut writer = BufWriter::new(file);

        for (index, instruction) in instructions.iter().enumerate() {
            writeln!(
                writer,
                "{:0padding$}  {}",
                index,
                instruction,
                padding = padding
            )?;
        }
    }

    Ok(())
}

// TODO: Remove this in favor of std's log10 once it gets stabilized.
fn log10(value: usize) -> usize {
    let zeros = value.leading_zeros() as usize;
    let lookup = unsafe { *LOG10_TABLE.get_unchecked(zeros) };
    (63 - zeros) * 77 / 256 + (value >= lookup) as usize
}

const LOG10_TABLE: [usize; 64] = [
    10000000000000000000,
    -1 as _,
    -1 as _,
    -1 as _,
    1000000000000000000,
    -1 as _,
    -1 as _,
    100000000000000000,
    -1 as _,
    -1 as _,
    10000000000000000,
    -1 as _,
    -1 as _,
    -1 as _,
    1000000000000000,
    -1 as _,
    -1 as _,
    100000000000000,
    -1 as _,
    -1 as _,
    10000000000000,
    -1 as _,
    -1 as _,
    -1 as _,
    1000000000000,
    -1 as _,
    -1 as _,
    100000000000,
    -1 as _,
    -1 as _,
    10000000000,
    -1 as _,
    -1 as _,
    -1 as _,
    1000000000,
    -1 as _,
    -1 as _,
    100000000,
    -1 as _,
    -1 as _,
    10000000,
    -1 as _,
    -1 as _,
    -1 as _,
    1000000,
    -1 as _,
    -1 as _,
    100000,
    -1 as _,
    -1 as _,
    10000,
    -1 as _,
    -1 as _,
    -1 as _,
    1000,
    -1 as _,
    -1 as _,
    100,
    -1 as _,
    -1 as _,
    10,
    -1 as _,
    -1 as _,
    -1 as _,
];
