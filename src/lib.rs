#![feature(iter_next_chunk, slice_as_chunks, const_trait_impl, effects, generic_const_exprs)]

#[macro_use]
extern crate disclose;

//use std::{num::NonZero, time::{Duration, SystemTime}};

#[disclose]
mod prelude {
    use super::{
        Rect, Input, Outcome, Success, Side, Vertical, Environment, Update, 
//        Room, House, 
//        room::Enemy
    };
    pub mod room {
//    	use crate::room::{SCREEN_WIDTH, SCREEN_HEIGHT, VERT_CEILING, VERT_FLOOR};
        type Id = std::num::NonZero<u16>;
    }
} 

mod cart;

pub use cart::{Point, Rect, Size};

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
    Score(u16),
    Life,
    Bands(u8),
    Energy(u8),
    Shoot,
    Zoom,
    Start(Environment),
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
//    Leave{score: u32, destination: Option<(prelude::room::Id, Entrance)>},
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Vertical {
    Down, Up,
}

//pub use room::Room;
//pub use house::House;
//pub use object::{Object, ObjectKind};
//pub use play::{Entrance, Play};

//mod object;
//mod room;
//mod house;

// mod play;

//mod import;
