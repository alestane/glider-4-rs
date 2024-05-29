#![feature(iter_next_chunk, slice_as_chunks, const_trait_impl, effects, generic_const_exprs)]

#[macro_use]
extern crate disclose;

use std::{num::NonZero, time::{Duration, SystemTime}};

#[disclose]
mod prelude {
    use super::{Rect, Input, Outcome, Success, Side, Vertical, Room, House};
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    _left: u16, 
    _top: u16, 
    _right: u16, 
    _bottom: u16,
}

impl Rect {
    pub const fn new(left: u16, top: u16, right: u16, bottom: u16) -> Self {
        let (left, top) = (left.min(right), top.min(bottom));
        Self {
            _left: left,
            _top: top,
            _right: right.max(left + 1),
            _bottom: bottom.max(top + 1),
        }
    }

    pub fn left  (&self) -> u16 { self._left   }
    pub fn top   (&self) -> u16 { self._top    }
    pub fn right (&self) -> u16 { self._right  }
    pub fn bottom(&self) -> u16 { self._bottom }

    pub fn width (&self) -> NonZero<u16> { unsafe{ NonZero::new_unchecked((self._right - self._left).max(1)) } }
    pub fn height(&self) -> NonZero<u16> { unsafe{ NonZero::new_unchecked((self._bottom - self._top).max(1)) } }

    pub fn x(&self) -> u16 { (self._left + self._right) / 2 }
    pub fn y(&self) -> u16 { (self._top + self._bottom) / 2 }
}

impl From<Rect> for (u16, u16, u16, u16) {
    fn from(value: Rect) -> Self {
        (value._left, value._top, value._right, value._bottom)
    }
}

impl From<(u16, u16, u16, u16)> for Rect {
    fn from(value: (u16, u16, u16, u16)) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
    }
}

pub enum Input {
    Left, 
    Right,
    Flip,
    Shoot,
}

pub enum Outcome {
    Continue,
    Dead,
    Leave{score: u16, destination: Option<u8>},
}

#[derive(Debug, Clone)]
pub struct Success {
    pub score: u32,
    pub level: u16,
    pub name: String,
    pub room: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Left, Right,
}

impl std::ops::Neg for Side {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Vertical {
    Down, Up, 
}

pub use room::Room;
pub use house::House; 
pub use object::{Object, ObjectKind};
pub use play::Entrance;

mod object;
mod room;
mod house;

mod play;

mod import;