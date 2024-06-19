#![feature(iter_next_chunk, slice_as_chunks, const_trait_impl, effects, generic_const_exprs)]

#[macro_use]
extern crate disclose;

use std::{num::NonZero, time::{Duration, SystemTime}};

#[disclose]
mod prelude {
    use super::{Rect, Input, Outcome, Success, Side, Vertical, Room, House, Environment, Update, room::Enemy};
    pub mod room {
    	use crate::room::{SCREEN_WIDTH, SCREEN_HEIGHT, VERT_CEILING, VERT_FLOOR};
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
	x_: i16,
	y_: i16,
}

impl Point {
	pub const fn new(x: i16, y: i16) -> Self { (x, y).into() }
	pub const fn x(&self) -> i16 { self.x_ }
	pub const fn y(&self) -> i16 { self.y_ }
	fn frame(&self, width: NonZero<u16>, height: NonZero<u16>) -> Option<Rect> {
		let left = self.x_.saturating_sub_unsigned(width.get() / 2);
		let top = self.y_.saturating_sub_unsigned(height.get() / 2);
		let right = left.saturating_add_unsigned(width.get());
		let bottom = top.saturating_add_unsigned(height.get());
		let (left_, top_, Some(right_), Some(bottom_)) = (
			0u16.saturating_add_signed(left),
			0u16.saturating_add_signed(top),
			0u16.checked_add_signed(right),
			0u16.checked_add_signed(bottom)
		) else { return None };
		Some(Rect {left_, top_, right_, bottom_})
	}
}

impl From<(i16, i16)> for Point {
	fn from((x_, y_): (i16, i16)) -> Self { Self {x_, y_} }
}

impl From<Point> for (i16, i16) {
	fn from(Point{x_, y_}: Point) -> Self { (x_, y_) }
}

impl std::ops::Neg for Point {
	type Output = Self;
	fn neg(self) -> Self { Self {x_: -self.x_, y_: -self.y_} }
}

impl<I: Into<Point>> std::ops::AddAssign<I> for Point {
	fn add_assign(&mut self, rhs: I) {
		let Point{x_, y_} = rhs.into();
		self.x_ += x_;
		self.y_ += y_;
	}
}

impl<I: Into<Point>> std::ops::Add<I> for Point {
	type Output = Self;
	fn add(mut self, rhs: I) -> Self::Output { self += rhs; self }
}

impl<I: Into<Point>> std::ops::Sub<I> for Point {
	type Output = Self;
	fn sub(self, rhs: I) -> Self::Output { let other: Point = rhs.into(); self + -other }
}

impl<I: Into<Point>> std::ops::Mul<I> for Point {
	type Output = Self;
	fn mul(mut self, rhs: I) -> Self::Output { self *= rhs; self }
}

impl std::ops::Mul<i16> for Point {
	type Output = Self;
	fn mul(mut self, rhs: i16) -> Self::Output { self *= rhs; self }
}

impl<I: Into<Point>> std::ops::MulAssign<I> for Point {
	fn mul_assign(&mut self, rhs: I) {
		let Point { x_, y_ } = rhs.into();
		self.x_ *= x_;
		self.y_ *= y_;
    }
}

impl std::ops::MulAssign<i16> for Point {
	fn mul_assign(&mut self, rhs: i16) {
    	let n: i16 = rhs.into();
    	*self *= (n, n);
	}
}

impl<I: Into<Point>> std::ops::Div<I> for Point {
	type Output = Self;
	fn div(mut self, rhs: I) -> Self::Output { self /= rhs; self }
}

impl std::ops::Div<i16> for Point {
	type Output = Self;
	fn div(mut self, rhs: i16) -> Self::Output { self /= rhs; self }
}

impl<I: Into<Point>> std::ops::DivAssign<I> for Point {
	fn div_assign(&mut self, rhs: I) {
		let Point { x_, y_ } = rhs.into();
		self.x_ /= x_;
		self.y_ /= y_;
    }
}

impl std::ops::DivAssign<i16> for Point {
	fn div_assign(&mut self, rhs: i16) {
    	let n: i16 = rhs.into();
    	*self /= (n, n);
	}
}

impl<I: Into<Point>> std::ops::SubAssign<I> for Point {
	fn sub_assign(&mut self, rhs: I) { let other: Point = rhs.into(); *self += -other; }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    left_: u16,
    top_: u16,
    right_: u16,
    bottom_: u16,
}

impl Rect {
    pub const fn new(left: u16, top: u16, right: u16, bottom: u16) -> Self {
        let (left_, top_) = (if left < right {left} else {right}, if top < bottom {top} else {bottom});
        let (right_, bottom_) = (if right == left_ {left_ + 1} else {right}, if bottom == top_ {top_ + 1} else {bottom});
        Self {
            left_, top_, right_, bottom_,
        }
    }

    pub const fn cropped_on(center: (u16, u16), width: u16, height: u16) -> Self {
        Rect{
            left_: center.0.saturating_sub(width / 2),
            top_: center.1.saturating_sub(height / 2),
            right_: center.0.saturating_add((width + 1) / 2),
            bottom_: center.1.saturating_add((height + 1) / 2),
        }
    }

    pub const fn clamped_on(center: (u16, u16), width: u16, height: u16) -> Self {
        let mut left_ = center.0.saturating_sub(width / 2);
        let mut top_ = center.1.saturating_sub(width / 2);
        let right_ = left_.saturating_add(width);
        let bottom_ = top_.saturating_add(height);
        left_ = left_.min(right_ - width);
        top_ = top_.min(bottom_ - height);
        Self {
            left_, top_, right_, bottom_
        }
    }

    pub fn left  (&self) -> u16 { self.left_   }
    pub fn top   (&self) -> u16 { self.top_    }
    pub fn right (&self) -> u16 { self.right_  }
    pub fn bottom(&self) -> u16 { self.bottom_ }

    pub fn width (&self) -> NonZero<u16> { unsafe{ NonZero::new_unchecked((self.right_ - self.left_).max(1)) } }
    pub fn height(&self) -> NonZero<u16> { unsafe{ NonZero::new_unchecked((self.bottom_ - self.top_).max(1)) } }

    pub fn x(&self) -> u16 { (self.left_ + self.right_) / 2 }
    pub fn y(&self) -> u16 { (self.top_ + self.bottom_) / 2 }
}

impl std::ops::BitAnd for Rect {
    type Output = Option<Self>;
    fn bitand(self, rhs: Self) -> Self::Output {
        let left_ = self.left_.max(rhs.left_);
        let right_ = self.right_.min(rhs.right_);
        let top_ = self.top_.max(rhs.top_);
        let bottom_ = self.bottom_.min(rhs.bottom_);
        ((left_ < right_) & (top_ < bottom_)).then_some(Self{left_, top_, right_, bottom_})
    }
}

impl std::ops::BitAnd<Option<Rect>> for Rect {
	type Output = Option<Rect>;
	fn bitand(self, rhs: Option<Rect>) -> Self::Output { rhs.and_then(|b| self & b) }
}

impl std::ops::BitAnd<Rect> for Option<Rect> {
	type Output = Option<Rect>;
	fn bitand(self, rhs: Rect) -> Self::Output { self.and_then(|a| a & rhs) }
}

impl From<Rect> for (u16, u16, u16, u16) {
    fn from(value: Rect) -> Self {
        (value.left_, value.top_, value.right_, value.bottom_)
    }
}

impl From<(u16, u16, u16, u16)> for Rect {
    fn from(value: (u16, u16, u16, u16)) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
    }
}

impl<I: Into<(i16, i16)>> std::ops::Shr<I> for Rect {
    type Output = Rect;
    fn shr(mut self, rhs: I) -> Self::Output {
        self >>= rhs.into();
        self
    }
}

impl<I: Into<(i16, i16)>> std::ops::ShrAssign<I> for Rect {
    fn shr_assign(&mut self, rhs: I) {
        let rhs = rhs.into();
        self.left_ = self.left_.saturating_add_signed(rhs.0);
        self.right_ = self.right_.saturating_add_signed(rhs.0);
        self.top_ = self.top_.saturating_add_signed(rhs.1);
        self.bottom_ = self.bottom_.saturating_add_signed(rhs.1);
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
    Leave{score: u32, destination: Option<(NonZero<u16>, Entrance)>},
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

pub use room::Room;
pub use house::House;
pub use object::{Object, ObjectKind};
pub use play::{Entrance, Play};

mod object;
mod room;
mod house;

mod play;

mod import;
