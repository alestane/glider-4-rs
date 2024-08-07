#![feature(
    iter_next_chunk, slice_as_chunks, iterator_try_collect, is_none_or,
    iter_advance_by, iter_collect_into, const_try,
    const_option,
    generic_arg_infer, generic_const_exprs, const_refs_to_cell
)]

#[macro_use]
extern crate disclose;

use std::ops::{Mul, Neg, Range};

#[disclose]
mod prelude {
    use std::num::NonZero; 

    pub use super::{
        Input, Outcome, Success, Side, Vertical, Environment, Update, play::Player, 
        Bounds, Position, Size, cart::{Span, Rise, Rect, Point},
        Object, Room, House, 
    };
    pub mod room {
    	pub use crate::room::{SCREEN_WIDTH, SCREEN_HEIGHT, VERT_CEILING, VERT_FLOOR};
        pub type Id = super::NonZero<u16>;
    }
    pub mod object {
        pub use crate::object::{Kind, Motion, Duct};
        pub type Id = super::NonZero<usize>;
    }
    pub use object::Duct::*;

    pub type Anchor = (Span, Rise);
    pub const TOPLEFT:      Anchor = (Span::Left, Rise::Top);
    pub const TOP:          Anchor = (Span::Center, Rise::Top);
    pub const TOPRIGHT:     Anchor = (Span::Right, Rise::Top);
    pub const LEFT:         Anchor = (Span::Left, Rise::Center);
    pub const CENTER:       Anchor = (Span::Center, Rise::Center);
    pub const RIGHT:        Anchor = (Span::Right, Rise::Center);
    pub const BOTTOMLEFT:   Anchor = (Span::Left, Rise::Bottom);
    pub const BOTTOM:       Anchor = (Span::Center, Rise::Bottom);
    pub const BOTTOMRIGHT:  Anchor = (Span::Right, Rise::Bottom);
} 

mod cart;

pub type Bounds = cart::Rect;
pub type Position = cart::Point;
pub type Interval = Range<i16>;
pub type Reference = cart::Point;
pub type Displacement = cart::Displacement;
pub type Size = cart::Size;

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
    Switch,
    Duct,
}

#[derive(Debug, Clone, Copy)]
pub enum Update {
    Score(u16, Position),
    Life(u16, Position),
    Bands(u8, Position),
    Energy(u8, Position),
    Shoot,
    Zoom,
    Start(Environment, Option<object::Id>),
    Bump,
    Fade(bool),
    Turn(Side),
    Lights,
    Air,
    Burn,
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Continue(Option<Vec<Update>>),
    Dead,
    Leave{score: u32, destination: Option<(prelude::room::Id, Entrance)>},
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

impl Mul<i16> for Side {
    type Output = i16;
    fn mul(self, rhs: i16) -> Self::Output {
        match self {
            Self::Left => -rhs,
            Self::Right => rhs,
        }
    }
}

impl Mul<Side> for Displacement {
    type Output = Displacement;
    fn mul(self, rhs: Side) -> Self::Output {
        Self::new(rhs * self.x(), self.y())
    }
}

impl Neg for Side {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

impl Neg for &Side {
    type Output = Side;
    fn neg(self) -> Self::Output { -*self }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vertical {
    Down, Up,
}

impl Neg for Vertical {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

impl Neg for &Vertical {
    type Output = Vertical;
    fn neg(self) -> Self::Output { -*self }
}

pub use room::Room;
pub use house::House;
pub use object::Object;
pub use play::{Entrance, Play};

mod object;
mod room;
mod house;

mod play;

mod import;
