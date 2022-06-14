/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use membrane::instruction::Instruction;
use membrane::interpreter::TapeSize;
use membrane::optimizer::optimize;
use membrane::parser::parse_string;

// Trivial runs of certain instructions should get squashed together.
#[test]
fn trivial_run_squash() {
    const INSTRUCTION_COUNT_MAX: usize = 5;

    let mut instructions = parse_string("+++++-->>><<<<<,....,,").expect("valid BF given");
    optimize(false, &mut instructions, TapeSize::Infinite);

    assert!(instructions.len() <= INSTRUCTION_COUNT_MAX);
}

// Loops should be removed if we can trivially prove their zero-test
// always fails (the relevant cell is always zero).
#[test]
fn trivial_loop_removal() {
    let mut instructions = parse_string("[.][[[>]]][[]]").expect("valid BF given");
    optimize(false, &mut instructions, TapeSize::Infinite);

    assert!(instructions.is_empty());
}

// We want to squash certain patterns of instructions into a primitive one,
// such as AddRelative. Make sure some recognition and substitution happens.
#[test]
fn simple_pattern_recognition() {
    let mut instructions = Vec::new();

    // [+]
    {
        instructions.extend_from_slice(&[
            Instruction::JumpIfZero { location: 0 },
            Instruction::Add(1),
            Instruction::JumpIfNotZero { location: 0 },
        ]);

        optimize(false, &mut instructions, TapeSize::Infinite);
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0], Instruction::SetValue(0));
        instructions.clear();
    }

    // [-]
    {
        instructions.extend_from_slice(&[
            Instruction::JumpIfZero { location: 0 },
            Instruction::Add(-1),
            Instruction::JumpIfNotZero { location: 0 },
        ]);

        optimize(false, &mut instructions, TapeSize::Infinite);
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0], Instruction::SetValue(0));
        instructions.clear();
    }

    // >>>-----<<<
    {
        instructions.extend_from_slice(&[
            Instruction::Move(3),
            Instruction::Add(-5),
            Instruction::Move(-3),
        ]);

        optimize(false, &mut instructions, TapeSize::Infinite);
        assert_eq!(instructions.len(), 1);
        assert_eq!(
            instructions[0],
            Instruction::AddRelative {
                offset: 3,
                amount: -5
            }
        );
        instructions.clear();
    }

    // <<<+++++>>>
    {
        instructions.extend_from_slice(&[
            Instruction::Move(-3),
            Instruction::Add(5),
            Instruction::Move(3),
        ]);

        optimize(false, &mut instructions, TapeSize::Infinite);
        assert_eq!(instructions.len(), 1);
        assert_eq!(
            instructions[0],
            Instruction::AddRelative {
                offset: -3,
                amount: 5
            }
        );
        instructions.clear();
    }

    // +++>++>>
    {
        instructions.extend_from_slice(&[
            Instruction::Add(3),
            Instruction::Move(1),
            Instruction::Add(2),
            Instruction::Move(2),
        ]);

        optimize(false, &mut instructions, TapeSize::Infinite);
        assert_eq!(instructions.len(), 1);
        assert_eq!(
            instructions[0],
            Instruction::AddVectorMove {
                stride: 3,
                vector: [3, 2, 0, 0],
            }
        );
        instructions.clear();
    }

    // [++>>>]
    {
        instructions.extend_from_slice(&[
            Instruction::JumpIfZero { location: 0 },
            Instruction::Add(2),
            Instruction::Move(3),
            Instruction::JumpIfNotZero { location: 0 },
        ]);

        optimize(false, &mut instructions, TapeSize::Infinite);
        assert_eq!(instructions.len(), 1);
        assert_eq!(
            instructions[0],
            Instruction::MoveRightToZero {
                increment: 2,
                stride: 3,
            }
        );
        instructions.clear();
    }

    // [++<<<]
    {
        instructions.extend_from_slice(&[
            Instruction::JumpIfZero { location: 0 },
            Instruction::Add(2),
            Instruction::Move(-3),
            Instruction::JumpIfNotZero { location: 0 },
        ]);

        optimize(false, &mut instructions, TapeSize::Infinite);
        assert_eq!(instructions.len(), 1);
        assert_eq!(
            instructions[0],
            Instruction::MoveLeftToZero {
                increment: 2,
                stride: 3,
            }
        );
        instructions.clear();
    }
}
