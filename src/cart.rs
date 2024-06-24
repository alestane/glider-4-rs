use std::{clone::Clone, cmp::{Eq, PartialEq}, fmt::Debug, marker::Copy, num::NonZero, ops::{Add, AddAssign, Mul}};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point<N: Sized> {
	x_: N,
	y_: N,
}

impl Default for Point<u16> {
    fn default() -> Self { Self {x_: 0, y_: 0} }
}

impl Default for Point<i16> {
    fn default() -> Self { Self {x_: 0, y_: 0} }
}

#[const_trait]
trait Narrow<N> {
    fn narrow(&self) -> N;
}

impl const Narrow<u16> for u16 {
    fn narrow(&self) -> u16 { *self }
}

impl const Narrow<u16> for i16 {
    fn narrow(&self) -> u16 { 0u16.saturating_add_signed(*self) }
}

impl const Narrow<i16> for u16 {
    fn narrow(&self) -> i16 { 0i16.saturating_add_unsigned(*self) }
}

impl const Narrow<i16> for i16 {
    fn narrow(&self) -> i16 { *self }
}

impl<N: Copy> Point<N> {
    pub const fn new(x: N, y: N) -> Self { Self{x_: x, y_: y} }
    pub const fn x(&self) -> N { self.x_ }
    pub const fn y(&self) -> N { self.y_ }
}

impl Point<u16> {
    pub(crate) const fn narrow<N: ~const Narrow<u16>>(value: &Point<N>) -> Self {
        Self::new(value.x_.narrow(), value.y_.narrow()) 
    }
}

impl Point<i16> {
    pub(crate) const fn narrow<N: ~const Narrow<i16>>(value: &Point<N>) -> Self {
        Self::new(value.x_.narrow(), value.y_.narrow()) 
    }   
}

impl TryFrom<Point<i16>> for Point<u16> {
    type Error = <u16 as TryFrom<i16>>::Error;
    fn try_from(value: Point<i16>) -> Result<Self, Self::Error> {
        Ok(Self{ x_: u16::try_from(value.x_)?, y_: u16::try_from(value.y_)? })
    }
}

impl TryFrom<Point<u16>> for Point<i16> {
    type Error = <i16 as TryFrom<u16>>::Error;
    fn try_from(value: Point<u16>) -> Result<Self, Self::Error> {
        Ok(Self{ x_: i16::try_from(value.x_)?, y_: i16::try_from(value.x_)?})
    }
}

impl<N: Copy> From<(N, N)> for Point<N> {
    fn from((x_, y_): (N, N)) -> Self { Self{x_, y_} }
}

impl<N: Copy> From<Point<N>> for (N, N) {
	fn from(Point{x_, y_}: Point<N>) -> Self { (x_, y_) }
}

impl std::ops::Neg for Point<i16> {
	type Output = Self;
	fn neg(self) -> Self { Self {x_: -self.x_, y_: -self.y_} }
}

impl<N: AddAssign<N>, I: Into<Point<N>>> AddAssign<I> for Point<N> {
	fn add_assign(&mut self, rhs: I) {
		let Point{x_, y_} = rhs.into();
		self.x_ += x_;
		self.y_ += y_;
	}
}

impl<N: Add<N, Output=N>, I: Into<Point<N>>> Add<I> for Point<N>  where Self: AddAssign<I>{ 
	type Output = Self;
	fn add(mut self, rhs: I) -> Self::Output { self += rhs; self }
}

impl<I: Into<Point<i16>>> std::ops::Sub<I> for Point<i16> {
	type Output = Self;
	fn sub(self, rhs: I) -> Self::Output { let other: Point<i16> = rhs.into(); self + -other }
}

impl<I: Into<Point<i16>>> std::ops::Mul<I> for Point<i16> {
	type Output = Self;
	fn mul(mut self, rhs: I) -> Self::Output { self *= rhs; self }
}

impl std::ops::Mul<i16> for Point<i16> {
	type Output = Self;
	fn mul(mut self, rhs: i16) -> Self::Output { self *= rhs; self }
}

impl<I: Into<Point<i16>>> std::ops::MulAssign<I> for Point<i16> {
	fn mul_assign(&mut self, rhs: I) {
		let Point { x_, y_ } = rhs.into();
		self.x_ *= x_;
		self.y_ *= y_;
    }
}

impl std::ops::MulAssign<i16> for Point<i16> {
	fn mul_assign(&mut self, rhs: i16) {
    	let n: i16 = rhs.into();
    	*self *= (n, n);
	}
}

impl<I: Into<Point<i16>>> std::ops::Div<I> for Point<i16> {
	type Output = Self;
	fn div(mut self, rhs: I) -> Self::Output { self /= rhs; self }
}

impl std::ops::Div<i16> for Point<i16> {
	type Output = Self;
	fn div(mut self, rhs: i16) -> Self::Output { self /= rhs; self }
}

impl<I: Into<Point<i16>>> std::ops::DivAssign<I> for Point<i16> {
	fn div_assign(&mut self, rhs: I) {
		let Point { x_, y_ } = rhs.into();
		self.x_ /= x_;
		self.y_ /= y_;
    }
}

impl std::ops::DivAssign<i16> for Point<i16> {
	fn div_assign(&mut self, rhs: i16) {
    	let n: i16 = rhs.into();
    	*self /= (n, n);
	}
}

impl<I: Into<Point<i16>>> std::ops::SubAssign<I> for Point<i16> {
	fn sub_assign(&mut self, rhs: I) { let other: Point<i16> = rhs.into(); *self += -other; }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    width_: NonZero<u16>,
    height_: NonZero<u16>,
}

impl Size {
    pub(crate) const fn new(width: u16, height: u16) -> Option<Self> { 
        match (NonZero::new(width), NonZero::new(height)) {
            (Some(width_), Some(height_)) => Some(Self{width_, height_}),
            _ => None,
        } 
    }
    pub(crate) const unsafe fn new_unchecked(width_: u16, height_: u16) -> Self { 
        let (width_, height_) =  (NonZero::new_unchecked(width_), NonZero::new_unchecked(height_));
        Self { width_, height_}
    }
    pub const fn width(&self) -> u16 { self.width_.get() }
    pub const fn height(&self) -> u16 { self.height_.get() }
}

impl From<(NonZero<u16>, NonZero<u16>)> for Size {
    fn from(value: (NonZero<u16>, NonZero<u16>)) -> Self { Self{width_: value.0, height_: value.1} }
}

pub(crate) enum Span {Left = -1, Center = 0, Right = 1}
pub(crate) enum Rise {Top = -1, Center = 0, Bottom = 1}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect<N: Debug + Clone + Copy + PartialEq + Eq + TryInto<u16>> {
    left_: N,
    top_: N,
    width_: NonZero<u16>,
    height_: NonZero<u16>,
}

impl Rect<u16> {
    pub(crate) const fn new(left: u16, top: u16, right: u16, bottom: u16) -> Option<Self> {
        let (Some(width_), Some(height_)) = (NonZero::new(right.abs_diff(left)), NonZero::new(bottom.abs_diff(top))) else {
            return None
        };
        let (left_, top_) = (if left < right {left} else {right}, if top < bottom {top} else {bottom});
        Some(Self {
            left_, top_, width_, height_,
        })
    }

    pub(crate) const unsafe fn new_unchecked(left: u16, top: u16, right: u16, bottom: u16) -> Self {
        let left_ = if right < left {right} else {left};
        let top_ = if bottom < top {bottom} else {top};
        let width_ = NonZero::new_unchecked(right.abs_diff(left));
        let height_ = NonZero::new_unchecked(bottom.abs_diff(top));
        Self {left_, top_, width_, height_}
    }

    pub(crate) fn cropped_on(center: (u16, u16), width: NonZero<u16>, height: NonZero<u16>) -> Option<Self> {
        let (width_, height_) = (width.get(), height.get());
        Some(Self{
            left_: center.0.saturating_sub(width_ / 2),
            top_: center.1.saturating_sub(height_ / 2),
            width_: NonZero::new(center.0.saturating_add(width_) - center.0)?,
            height_: NonZero::new(center.1.saturating_add(height_) - center.1)?,
        })
    }

    pub(crate) const fn clamped_on(center: (u16, u16), width: NonZero<u16>, height: NonZero<u16>) -> Self {
        let (width_, height_) = (width, height);
        let (width, height) = (width_.get(), height_.get());
        let mut left_ = center.0.saturating_sub(width / 2);
        let mut top_ = center.1.saturating_sub(width / 2);
        let right_ = left_.saturating_add(width);
        let bottom_ = top_.saturating_add(height);
        left_ = left_.min(right_ - width);
        top_ = top_.min(bottom_ - height);
        Self {
            left_, top_, width_, height_
        }
    }

    pub const fn left  (&self) -> u16 { self.left_   }
    pub const fn top   (&self) -> u16 { self.top_    }
    pub const fn right (&self) -> u16 { self.left_ + self.width_.get() }
    pub const fn bottom(&self) -> u16 { self.top_ + self.height_.get()  }

    pub const fn width (&self) -> NonZero<u16> { self.width_ }
    pub const fn height(&self) -> NonZero<u16> { self.height_ }

    pub const fn size(&self) -> Size { Size{width_: self.width_, height_: self.height_} }

    pub const fn x(&self) -> u16 { self.left_ + self.width_.get() / 2 }
    pub const fn y(&self) -> u16 { self.top_ + self.height_.get() / 2 }

    pub const fn center(&self) -> Point<u16> {Point::new( self.x(), self.y() )}
}

impl Default for Rect<u16> {
    fn default() -> Self {
        unsafe { Rect{left_: 0, top_: 0, width_: NonZero::new_unchecked(1), height_: NonZero::new_unchecked(1)} }
    }
}

impl std::ops::BitAnd for Rect<u16> {
    type Output = Option<Self>;
    fn bitand(self, rhs: Self) -> Self::Output {
        let left_ = self.left_.max(rhs.left_);
        let right_ = self.right().min(rhs.right());
        let top_ = self.top_.max(rhs.top_);
        let bottom_ = self.bottom().min(rhs.bottom());
        ((left_ < right_) & (top_ < bottom_)).then_some(unsafe{Self{left_, top_, width_: NonZero::new_unchecked(right_ - left_), height_: NonZero::new_unchecked(bottom_ - top_)}})
    }
}

impl std::ops::BitAnd<Option<Rect<u16>>> for Rect<u16> {
	type Output = Option<Rect<u16>>;
	fn bitand(self, rhs: Option<Self>) -> Self::Output { rhs.and_then(|b| self & b) }
}

impl std::ops::BitAnd<Rect<u16>> for Option<Rect<u16>> {
	type Output = Option<Rect<u16>>;
	fn bitand(self, rhs: Rect<u16>) -> Self::Output { self.and_then(|a| a & rhs) }
}

impl From<Rect<u16>> for (u16, u16, u16, u16) {
    fn from(value: Rect<u16>) -> Self {
        (value.left_, value.top_, value.right(), value.bottom())
    }
}

impl From<(u16, u16, NonZero<u16>, NonZero<u16>)> for Rect<u16> {
    fn from((left_, top_, width_, height_): (u16, u16, NonZero<u16>, NonZero<u16>)) -> Self {
        Self{left_, top_, width_, height_}
    }
}

impl TryFrom<(u16, u16, u16, u16)> for Rect<u16> {
    type Error = ();
    fn try_from(value: (u16, u16, u16, u16)) -> Result<Self, Self::Error> {
        Self::new(value.0, value.1, value.2, value.3).ok_or(())
    }
}

impl<I: Into<(i16, i16)>> std::ops::Shr<I> for Rect<u16> {
    type Output = Self;
    fn shr(mut self, rhs: I) -> Self::Output {
        self >>= rhs.into();
        self
    }
}

impl<I: Into<(i16, i16)>> std::ops::ShrAssign<I> for Rect<u16> {
    fn shr_assign(&mut self, rhs: I) {
        let rhs = rhs.into();
        let (width, height) = (self.width_.get(), self.height_.get());
        self.left_ = self.left_.saturating_add_signed(rhs.0).min(self.left_.saturating_add(width) - width);
        self.top_ = self.top_.saturating_add_signed(rhs.1).min(self.top_.saturating_add(height) - height);
    }
}

impl Mul<(Span, Rise)> for Rect<u16> {
    type Output = Point<u16>;
    fn mul(self, (h, v): (Span, Rise)) -> Self::Output {
        let x_ = match h {
            Span::Left => self.left(),
            Span::Center => self.x(),
            Span::Right => self.right(),
        };
        let y_ = match v {
            Rise::Top => self.top(),
            Rise::Center => self.y(),
            Rise::Bottom => self.bottom(),
        };
        Self::Output{x_, y_}
    }
}

impl Mul<(Rise, Span)> for Rect<u16> {
    type Output = Point<u16>;
    fn mul(self, (v, h): (Rise, Span)) -> Self::Output {
        self * (h, v)
    }
}