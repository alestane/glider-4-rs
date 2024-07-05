#[derive(Debug, Clone, Copy)]
pub struct Point {
	x_: i32,
	y_: i32,
}

impl Default for Point {
	fn default() -> Self { Self{x_: 0, y_:0} }
}

impl From<sdl2::rect::Point> for Point {
	fn from(value: sdl2::rect::Point) -> Self { Self {x_: value.x(), y_: value.y()}}
}

impl From<glider::Position> for Point {
	fn from(value: glider::Position) -> Self { Self{x_: value.x() as i32, y_: value.y() as i32} }
}

impl From<(i16, i16)> for Point {
	fn from((x, y): (i16, i16)) -> Self { Self{x_: x as i32, y_: y as i32} }
}

impl From<(i32, i32)> for Point {
	fn from((x_, y_): (i32, i32)) -> Self { Self{x_, y_} }
}

impl From<Point> for sdl2::rect::Point {
	fn from(Point{x_, y_}: Point) -> Self { Self::new(x_, y_) }
}

impl From<Point> for glider::Position {
	fn from(Point{x_, y_}: Point) -> Self { Self::new(x_ as u16, y_ as u16) }
}

impl From<Point> for (i32, i32) {
	fn from(Point{x_, y_}: Point) -> Self { (x_, y_) }
}

impl From<Point> for (i16, i16) {
	fn from(Point{x_, y_}: Point) -> Self { (x_ as i16, y_ as i16) }
}

#[derive(Debug, Clone, Copy)]
pub enum Rect {
    Unsigned(u32, u32, u32, u32),
    Signed(i32, i32, i32, i32),
//    Float(f32, f32, f32, f32),
}

impl Rect {
    pub const fn new_signed(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        let (left, top, right, bottom) = (
            if left < right {left} else {right}, if top < bottom {top} else {bottom}, if right > left {right} else {left}, if bottom > top {bottom} else {top}
        );
        Self::Signed(left, top, if right > left + 1 {right} else {left + 1}, if bottom > top + 1 {bottom} else {top + 1})
    }

    pub const fn new_unsigned(left: u32, top: u32, right: u32, bottom: u32) -> Self {
        let (left, top, right, bottom) = (
            if left < right {left} else {right}, if top < bottom {top} else {bottom}, if right > left {right} else {left}, if bottom > top {bottom} else {top}
        );
        Self::Unsigned(left, top, if right > left + 1 {right} else {left + 1}, if bottom > top + 1 {bottom} else {top + 1})
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::Signed(0, 0, 1, 1)
    }
}

impl From<(glider::Position, glider::Size)> for Rect {
    fn from((corner, size): (glider::Position, glider::Size)) -> Self {
        Self::new_unsigned(corner.x() as u32, corner.y() as u32, (corner.x() + size.width()) as u32, (corner.y() + size.height()) as u32)
    }
}

impl From<glider::Bounds> for Rect {
    fn from(value: glider::Bounds) -> Self {
        let (left, top, right, bottom) = value.into();
        Self::Unsigned(left as u32, top as u32, right as u32, bottom as u32)
    }
}

impl From<glider::prelude::Rect<i16>> for Rect {
    fn from(value: glider::prelude::Rect<i16>) -> Self {
        Self::new_signed(value.left() as i32, value.top() as i32, value.right() as i32, value.bottom() as i32)
    }
}

impl From<sdl2::rect::Rect> for Rect {
    fn from(value: sdl2::rect::Rect) -> Self {
        let (left, top, width, height) = value.into();
        Self::Signed(left, top, left.saturating_add_unsigned(width.max(1)), top.saturating_add_unsigned(height.max(1)))
    }
}

impl From<Rect> for glider::Bounds {
    fn from(value: Rect) -> Self {
        match value {
            Rect::Unsigned(l, t, r, b) => {
                let (left, top, right, bottom) = (
                    l.try_into().unwrap_or(u16::MAX - 1),
                    t.try_into().unwrap_or(u16::MAX - 1),
                    r.try_into().unwrap_or(u16::MAX),
                    b.try_into().unwrap_or(u16::MAX)
                );
                Self::new(left, top, right.max(left + 1), bottom.max(top + 1)).unwrap()
            }
            Rect::Signed(l, t, r, b) => {
                Self::new(
                    if l < 0 { 0u16 } else { l.try_into().unwrap_or(u16::MAX) },
                    if t < 0 { 0u16 } else { t.try_into().unwrap_or(u16::MAX) },
                    if r < 0 { 1u16 } else { r.try_into().unwrap_or(u16::MAX) },
                    if b < 0 { 1u16 } else { b.try_into().unwrap_or(u16::MAX) },
                ).unwrap()
            }
        }
    }
}

impl From<Rect> for sdl2::rect::Rect {
    fn from(value: Rect) -> Self {
        match value {
            Rect::Unsigned(l, t, r, b) => {
                Self::new(
                    l.try_into().unwrap_or(i32::MAX - 1),
                    t.try_into().unwrap_or(i32::MAX - 1),
                    r - l,
                    b - t
                )
            }
            Rect::Signed(l, t, r, b) => {
                Self::new(
                    l,
                    t,
                    r.abs_diff(l),
                    b.abs_diff(t)
                )
            }
        }
    }
}

impl From<Rect> for Option<sdl2::rect::Rect> {
    fn from(value: Rect) -> Self {
        Some(value.into())
    }
}

use std::ops::{Shl, Shr};

impl Shl<Point> for Rect {
    type Output = Rect;
    fn shl(self, Point{x_, y_}: Point) -> Self::Output {
        match self {
            Self::Signed(l, t, r, b) => Self::new_signed(l + x_, t + y_, r + x_, b + y_),
            Self::Unsigned(l, t, r, b) 
                => Self::new_unsigned(
                    l.saturating_add_signed(x_), 
                    t.saturating_add_signed(y_), 
                    r.saturating_add_signed(x_), 
                    b.saturating_add_signed(y_)
                )
        }
    }
}

impl Shr<Point> for Rect {
    type Output = Rect;
    fn shr(self, Point{x_, y_}: Point) -> Self::Output {
        match self {
            Self::Signed(l, t, r, b) => Self::new_signed(l - x_, t - y_, r - x_, b - y_),
            Self::Unsigned(l, t, r, b) 
                => Self::new_unsigned(
                    l.saturating_add_signed(-x_), 
                    t.saturating_add_signed(-y_), 
                    r.saturating_add_signed(-x_), 
                    b.saturating_add_signed(-y_)
                )
        }
    }
}
