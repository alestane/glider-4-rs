#![feature(iter_next_chunk, slice_as_chunks, const_trait_impl, effects, generic_const_exprs)]

#[macro_use]
extern crate disclose;

use std::{num::NonZero, time::{Duration, SystemTime}};

#[disclose]
mod prelude {
    use super::{Rect, Input, Outcome, Success, Side, Vertical, Room, House, Environment, Update};
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub const fn cropped_on(center: (u16, u16), width: u16, height: u16) -> Self {
        Rect{
            _left: center.0.saturating_sub(width / 2), 
            _top: center.1.saturating_sub(height / 2),
            _right: center.0.saturating_add((width + 1) / 2),
            _bottom: center.1.saturating_add((height + 1) / 2),
        }
    }

    pub const fn clamped_on(center: (u16, u16), width: u16, height: u16) -> Self {
        let mut _left = center.0.saturating_sub(width / 2);
        let mut _top = center.1.saturating_sub(width / 2);
        let _right = _left.saturating_add(width);
        let _bottom = _top.saturating_add(height);
        _left = _left.min(_right - width);
        _top = _top.min(_bottom - height);
        Self {
            _left, _top, _right, _bottom
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

impl std::ops::BitAnd for Rect {
    type Output = Option<Self>;
    fn bitand(self, rhs: Self) -> Self::Output {
        let _left = self._left.max(rhs._left);
        let _right = self._right.min(rhs._right);
        let _top = self._top.max(rhs._top);
        let _bottom = self._bottom.min(rhs._bottom);
        ((_left < _right) & (_top < _bottom)).then_some(Self{_left, _top, _right, _bottom})
    }
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

#[derive(Debug, Clone, Copy)]
pub enum Input {
    Go(Side),
    Flip,
    Shoot,
    Zoom,
}

#[derive(Debug, Clone, Copy)]
pub enum Environment {
    Ball,
    Outlet,
    Fish,
    Drip, 
    Guitar,
    Toast,
    Grease,
}

#[derive(Debug, Clone, Copy)]
pub enum Update {
    Score(u32),
    Life,
    Bands(u8),
    Energy(u8),
    Shoot,
    Zoom,
    Start(Environment),
    Bump,
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Continue(Option<Vec<Update>>),
    Dead,
    Leave{score: u32, destination: Option<u16>},
}

#[derive(Debug, Clone)]
pub struct Success {
    pub score: u32,
    pub level: u16,
    pub name: String,
    pub room: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left, Right,
}

impl std::ops::Mul<i16> for Side {
    type Output = i16;
    fn mul(self, rhs: i16) -> Self::Output {
        match self {
            Self::Left => -rhs,
            Self::Right => rhs,
        }
    }
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
pub use play::{Entrance, Play};

mod object;
mod room;
mod house;

mod play;

mod import;