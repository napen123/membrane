/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt;
use std::fmt::Formatter;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Instruction {
    Add(i8),
    Move(isize),
    Write(usize),
    Read(usize),
    JumpIfZero { location: usize },
    JumpIfNotZero { location: usize },

    SetValue(i8),

    AddRelative { offset: isize, amount: i8 },
    AddVector { vector: [i8; 4] },

    MoveRightToZero { increment: i8, stride: usize },
    MoveLeftToZero { increment: i8, stride: usize },
}

impl Instruction {
    #[inline]
    pub const fn preserves_tape_head(&self) -> bool {
        !matches!(
            self,
            Self::Move(_) | Self::MoveRightToZero { .. } | Self::MoveLeftToZero { .. }
        )
    }

    #[inline]
    pub const fn is_add_friendly(&self) -> bool {
        !matches!(self, Self::Read(_) | Self::SetValue(_))
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add(amount) => write!(f, "{:16}{:+}", "Add", amount),
            Self::Move(amount) => write!(
                f,
                "{:16}{}{}",
                "Move",
                if amount.is_positive() { '>' } else { '<' },
                amount.unsigned_abs()
            ),
            Self::Write(amount) => write!(f, "{:16}.{}", "Write", amount),
            Self::Read(amount) => write!(f, "{:16},{}", "Read", amount),
            Self::JumpIfZero { location } => write!(f, "{:16}[{}", "JumpIfZero", location),
            Self::JumpIfNotZero { location } => write!(f, "{:16}]{}", "JumpIfNotZero", location),

            Self::SetValue(value) => write!(f, "{:16}{}", "SetValue", value),
            Self::AddRelative { offset, amount } => {
                write!(f, "{:16}{:+}~{:+}", "AddRelative", offset, amount)
            }
            Self::AddVector { vector } => {
                write!(f, "{:16}{:?}", "AddVector", vector)
            }
            Self::MoveRightToZero { increment, stride } => {
                write!(f, "{:16}{:+}>{}", "MoveToZero", increment, stride)
            }
            Self::MoveLeftToZero { increment, stride } => {
                write!(f, "{:16}{:+}<{}", "MoveToZero", increment, stride)
            }
        }
    }
}
