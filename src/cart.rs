use std::{
    clone::Clone, cmp::{Eq, PartialEq}, fmt::Debug, marker::Copy, 
    num::NonZero,
    ops::{Deref, DerefMut, Add, AddAssign, Sub, SubAssign, DivAssign, Div, MulAssign, Mul, ShlAssign, Shl, ShrAssign, Shr, Neg, BitAnd}
};

use crate::Side;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Displacement {
    x_: i16,
    y_: i16,
}

impl Displacement {
    pub const fn new(x_: i16, y_: i16) -> Self { Self{x_, y_} }
    pub const fn x(&self) -> i16 { self.x_ }
    pub const fn y(&self) -> i16 { self.y_ }
    pub fn x_ref(&self) -> &i16 { &self.x_ }
    pub fn y_ref(&self) -> &i16 { &self.y_ }
    pub fn x_mut(&mut self) -> &mut i16 { &mut self.x_ }
    pub fn y_mut(&mut self) -> &mut i16 { &mut self.y_ }
    
    pub fn as_ref(&self) -> (&i16, &i16) { (&self.x_, &self.y_) }
    pub fn as_mut(&mut self) -> (&mut i16, &mut i16) { (&mut self.x_, &mut self.y_) }
}

impl Neg for Displacement {
    type Output = Self;
    fn neg(self) -> Self::Output { Self{x_: -self.x_, y_: -self.y_} }
}

impl<I: Into<i16>> From<(I, I)> for Displacement {
    fn from((x_, y_): (I, I)) -> Self { Self{x_: x_.into(), y_: y_.into()} }
}

impl<I: From<i16>> From<Displacement> for (I, I) {
    fn from(value: Displacement) -> Self { (value.x_.into(), value.y_.into()) }
}

impl AddAssign for Displacement {
    fn add_assign(&mut self, rhs: Self) {
        self.x_ += rhs.x_;
        self.y_ += rhs.y_;
    }
}

impl AddAssign<(i16, i16)> for Displacement {
    fn add_assign(&mut self, rhs: (i16, i16)) { 
        self.x_ += rhs.0;
        self.y_ += rhs.1;
     }
}

impl<T> Add<T> for Displacement where Displacement: AddAssign<T> {
    type Output = Self;
    fn add(mut self, rhs: T) -> Self::Output {
        self += rhs;
        self
    }    
}    

impl SubAssign for Displacement {
    fn sub_assign(&mut self, rhs: Self) {
        self.x_ -= rhs.x_;
        self.y_ -= rhs.y_;
    }
}

impl SubAssign<(i16, i16)> for Displacement {
    fn sub_assign(&mut self, rhs: (i16, i16)) { 
        
        self.x_ -= rhs.0;
        self.y_ -= rhs.1;
     }
}

impl<T> Sub<T> for Displacement where Self: SubAssign<T> {
    type Output = Self;
    fn sub(mut self, rhs: T) -> Self::Output {
        self -= rhs;
        self
    }
}

impl MulAssign for Displacement {
    fn mul_assign(&mut self, rhs: Self) {
        self.x_ *= rhs.x_;
        self.y_ *= rhs.y_;
    }
}

impl MulAssign<(i16, i16)> for Displacement {
    fn mul_assign(&mut self, rhs: (i16, i16)) {
        self.x_ = self.x_.saturating_mul(rhs.0);
        self.y_ = self.y_.saturating_mul(rhs.1);
    }
}

impl MulAssign<i16> for Displacement {
    fn mul_assign(&mut self, rhs: i16) {
        *self *= (rhs, rhs);
    }
}
impl MulAssign<u16> for Displacement {
    fn mul_assign(&mut self, rhs: u16) {
        *self *= (rhs as i16, rhs as i16);
    }
}

impl<T> Mul<T> for Displacement where Self: MulAssign<T> {
    type Output = Self;
    fn mul(mut self, rhs: T) -> Self::Output {
        self *= rhs;
        self
    }
}

impl DivAssign for Displacement {
    fn div_assign(&mut self, rhs: Self) {
        self.x_ /= rhs.x_;
        self.y_ /= rhs.y_;
    }
}

impl DivAssign<(i16, i16)> for Displacement {
    fn div_assign(&mut self, rhs: (i16, i16)) {
        self.x_ /= rhs.0;
        self.y_ /= rhs.1;
    }
}

impl DivAssign<i16> for Displacement {
    fn div_assign(&mut self, rhs: i16) {
        self.x_ /= rhs;
        self.y_ /= rhs;
    }
}

impl DivAssign<u16> for Displacement {
    fn div_assign(&mut self, rhs: u16) {
        *self /= (rhs as i16, rhs as i16);
    }
}

impl<T> Div<T> for Displacement
where
    Self: DivAssign<T>
{
    type Output = Self;
    fn div(mut self, rhs: T) -> Self::Output {
        self /= rhs;
        self
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Point (Displacement);

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.0.x_)
            .field("y", &self.0.y_)
            .finish()
    }
}

impl Deref for Point {
    type Target = Displacement;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Point {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Point {
    pub const fn new(x: i16, y: i16) -> Self { Self(Displacement::new(x, y)) }
}

impl<T> AddAssign<T> for Point where Displacement: AddAssign<T> {
    fn add_assign(&mut self, rhs: T) { self.0 += rhs }
}

impl<T> Add<T> for Point where Displacement: Add<T, Output = Displacement> {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output { Self(self.0 + rhs) }
}

impl<T> SubAssign<T> for Point where Displacement: SubAssign<T> {
    fn sub_assign(&mut self, rhs: T) { self.0 -= rhs }
}

impl<T> Sub<T> for Point where Displacement: Sub<T, Output = Displacement> {
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output { Self(self.0 - rhs) }
}

impl From<i16> for Point {
    fn from(value: i16) -> Self { Self(Displacement{x_: value, y_: value}) }
}

impl From<(i16, i16)> for Point {
    fn from((x_, y_): (i16, i16)) -> Self { Self(Displacement{x_, y_}) }
}

impl From<Point> for (i16, i16) {
	fn from(Point(Displacement{x_, y_}): Point) -> Self { (x_, y_) }
}

/* 

impl<T: Transfer<Signed = T> + Neg<Output = T>> Neg for Point<T> {
	type Output = Self;
	fn neg(self) -> Self { Self {x_: -self.x_, y_: -self.y_} }
}

impl<N: AddAssign, I: Into<Point<N>>> AddAssign<I> for Point<N> {
	fn add_assign(&mut self, rhs: I) {
		let Point{x_, y_} = rhs.into();
		self.x_ += x_;
		self.y_ += y_;
	}
}

impl<N: Add<Output=N>, I: Into<Point<N>>> Add<I> for Point<N>  where Self: AddAssign<I>{ 
	type Output = Self;
	fn add(mut self, rhs: I) -> Self::Output { self += rhs; self }
}

impl<N: SubAssign, I: Into<Point<N>>> SubAssign<I> for Point<N> {
	fn sub_assign(&mut self, rhs: I) {
		let Point{x_, y_} = rhs.into();
		self.x_ -= x_;
		self.y_ -= y_;
	}
}

impl<N: Sub<Output=N>, I: Into<Point<N>>> Sub<I> for Point<N> where Self: SubAssign<I>{
	type Output = Self;
	fn sub(mut self, rhs: I) -> Self::Output { self -= rhs; self }
}

impl<N: MulAssign, I: Into<Point<N>>> MulAssign<I> for Point<N> {
    fn mul_assign(&mut self, rhs: I) {
        let Point{x_, y_} = rhs.into();
        self.x_ *= x_;
        self.y_ *= y_;
    }
}

impl<N: Mul<Output = N>, I: Into<Point<N>>> Mul<I> for Point<N> where Self: MulAssign<I> {
    type Output = Self;
    fn mul(mut self, rhs: I) -> Self::Output { self *= rhs; self }
}

impl<N: DivAssign, I: Into<Point<N>>> DivAssign<I> for Point<N> {
    fn div_assign(&mut self, rhs: I) {
        let Point{x_, y_} = rhs.into();
        self.x_ /= x_;
        self.y_ /= y_;
    }
}

impl<N: Div<Output = N>, I: Into<Point<N>>> Div<I> for Point<N> where Self: DivAssign<I> {
	type Output = Self;
	fn div(mut self, rhs: I) -> Self::Output { self /= rhs; self }
} */

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    width_: NonZero<u16>,
    height_: NonZero<u16>,
}

impl Size {
    pub const fn new(width: u16, height: u16) -> Option<Self> { 
        Some(Self{
            width_: match NonZero::new(width) {Some(w) => w, None => return None}, 
            height_: match NonZero::new(height) {Some(h) => h, None => return None}
        })
    }
    pub const unsafe fn new_unchecked(width_: u16, height_: u16) -> Self { 
        let (width_, height_) =  (NonZero::new_unchecked(width_), NonZero::new_unchecked(height_));
        Self { width_, height_}
    }
    pub const fn width(&self) -> u16 { self.width_.get() }
    pub const fn height(&self) -> u16 { self.height_.get() }
}

impl Default for Size {
    fn default() -> Self { const{ Self{width_: NonZero::new(1).unwrap(), height_: NonZero::new(1).unwrap()} } }
}

impl AddAssign<(u16, u16)> for Size {
    fn add_assign(&mut self, rhs: (u16, u16)) { 
        self.width_ = NonZero::new(self.width_.get() + rhs.0).unwrap();
        self.height_ = NonZero::new(self.height_.get() + rhs.1).unwrap();
     }
}

impl Add<(u16, u16)> for Size {
    type Output = Self;
    fn add(mut self, rhs: (u16, u16)) -> Self::Output {
        self += rhs;
        self
    }
}

impl SubAssign<(u16, u16)> for Size {
    fn sub_assign(&mut self, rhs: (u16, u16)) { 
        
        self.width_ = unsafe{ NonZero::new_unchecked((self.width_.get() - rhs.0).max(1)) };
        self.height_ = unsafe{ NonZero::new_unchecked((self.height_.get() - rhs.0).max(1)) };
     }
}

impl Sub<(u16, u16)> for Size {
    type Output = Self;
    fn sub(mut self, rhs: (u16, u16)) -> Self::Output {
        self -= rhs;
        self
    }
}

impl MulAssign<(NonZero<u16>, NonZero<u16>)> for Size {
    fn mul_assign(&mut self, rhs: (NonZero<u16>, NonZero<u16>)) {
        self.width_ = self.width_.saturating_mul(rhs.0);
        self.height_ = self.height_.saturating_mul(rhs.1);
    }
}

impl MulAssign<NonZero<u16>> for Size {
    fn mul_assign(&mut self, rhs: NonZero<u16>) {
        *self *= (rhs, rhs);
    }
}

impl DivAssign<(NonZero<u16>, NonZero<u16>)> for Size {
    fn div_assign(&mut self, rhs: (NonZero<u16>, NonZero<u16>)) {
        self.width_ = NonZero::new(self.width_.get() / rhs.0.get()).unwrap_or(const{ NonZero::new(1).unwrap() });
        self.height_ = NonZero::new(self.height_.get() / rhs.1.get()).unwrap_or(const{ NonZero::new(1).unwrap() });
    }
}

impl DivAssign<NonZero<u16>> for Size {
    fn div_assign(&mut self, rhs: NonZero<u16>) {
        *self /= (rhs, rhs);
    }
}

impl<T> Div<T> for Size
where
    Self: DivAssign<T>
{
    type Output = Self;
    fn div(mut self, rhs: T) -> Self::Output {
        self /= rhs;
        self
    }
}

impl From<(NonZero<u16>, NonZero<u16>)> for Size {
    fn from(value: (NonZero<u16>, NonZero<u16>)) -> Self { 
        Self{
            width_: NonZero::new(value.0.get()).unwrap(), 
            height_: NonZero::new(value.1.get()).unwrap()
        } 
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Span {Left = -1, Center = 0, Right = 1}
#[derive(Debug, Clone, Copy)]
pub enum Rise {Top = -1, Center = 0, Bottom = 1}

impl From<Side> for Span {
    fn from(value: Side) -> Self { match value {Side::Left => Span::Left, Side::Right => Span::Right} }
}

impl Neg for Span {
    type Output = Self;
    fn neg(self) -> Self::Output { match self {Self::Left => Self::Right, Self::Right => Self::Left, _ => Self::Center} }
}

impl Neg for &Span {
    type Output = Span;
    fn neg(self) -> Self::Output { -*self }
}

impl Neg for Rise {
    type Output = Self;
    fn neg(self) -> Self::Output { match self {Self::Top => Self::Bottom, Self::Bottom => Self::Top, _ => Rise::Center} }
}

impl Neg for &Rise {
    type Output = Rise;
    fn neg(self) -> Self::Output { -*self }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect
{
    topleft_: Point,
    size_: Size,
}

impl Rect {
    pub const fn new(left: i16, top: i16, right: i16, bottom: i16) -> Option<Self> {
        Some(Self {
            topleft_: Point::new(if left < right {left} else {right}, if top < bottom {top} else {bottom}),
            size_: if let Some(s) = Size::new(left.abs_diff(right), top.abs_diff(bottom)) {s} else {return None}
        })
    }

    pub(crate) const unsafe fn new_unchecked(left: i16, top: i16, right: i16, bottom: i16) -> Self {
        Self{
            topleft_: Point::new(if left < right {left} else {right}, if top < bottom {top} else {bottom}),
            size_: Size::new_unchecked(left.abs_diff(right), top.abs_diff(bottom))
        }
    }

    pub const fn left  (&self) -> i16 { self.topleft_.0.x_   }
    pub const fn top   (&self) -> i16 { self.topleft_.0.y_    }
    pub const fn right (&self) -> i16 { self.topleft_.0.x_.saturating_add_unsigned(self.size_.width_.get()) }
    pub const fn bottom(&self) -> i16 { self.topleft_.0.y_.saturating_add_unsigned(self.size_.height_.get()) }

    pub const fn width (&self) -> NonZero<u16> { self.size_.width_ }
    pub const fn height(&self) -> NonZero<u16> { self.size_.height_ }

    pub const fn size(&self) -> Size{ self.size_ }
}

impl Rect {
    pub const fn x(&self) -> i16 { self.topleft_.0.x_.saturating_add_unsigned(self.size_.width_.get() / 2) }
    pub const fn y(&self) -> i16 { self.topleft_.0.y_.saturating_add_unsigned(self.size_.height_.get() / 2) }

    pub const fn center(&self) -> Point { Point::new( self.x(), self.y() ) }
}

impl From<(Point, Size)> for Rect {
    fn from((corner, size): (Point, Size)) -> Self {
        Self{ topleft_: corner, size_: size}
    }
}

impl BitAnd for Rect {
    type Output = Option<Self>;
    fn bitand(self, rhs: Self) -> Self::Output {
        let (left_, right_) = (self.left().max(rhs.left()), self.right().min(rhs.right()));
        let (top_, bottom_) = (self.top().max(rhs.top()), self.bottom().min(rhs.bottom()));
        if (left_ < right_) && (top_ < bottom_) {
            Some(Self{
                topleft_: Point::new(left_, top_), 
                size_: unsafe{ Size::new_unchecked(right_.abs_diff(left_), bottom_.abs_diff(top_)) }}
            )
        } else {
            None
        }
    }
}

impl BitAnd<Option<Rect>> for Rect {
	type Output = Option<Rect>;
	fn bitand(self, rhs: Option<Self>) -> Self::Output { rhs.and_then(|b| self & b) }
}

impl BitAnd<Rect> for Option<Rect> {
	type Output = Option<Rect>;
	fn bitand(self, rhs: Rect) -> Self::Output { self.and_then(|a| a & rhs) }
}

impl From<Rect> for (i16, i16, i16, i16) {
    fn from(value: Rect) -> Self {
        (value.left(), value.top(), value.right(), value.bottom())
    }
}

impl From<(i16, i16, NonZero<u16>, NonZero<u16>)> for Rect {
    fn from((left_, top_, width_, height_): (i16, i16, NonZero<u16>, NonZero<u16>)) -> Self {
        Self{topleft_: Point::new(left_, top_), size_: (width_, height_).into()}
    }
}

impl TryFrom<(i16, i16, i16, i16)> for Rect  {
    type Error = (u16, u16);
    fn try_from(value: (i16, i16, i16, i16)) -> Result<Self, Self::Error> {
        let (left_, width_) = (value.0.min(value.2), value.0.abs_diff(value.2));
        let (top_, height_) = (value.1.min(value.3), value.1.abs_diff(value.3));
        if left_ != value.0 || top_ != value.1 { return Err((width_, height_)) }
        Ok(Self{topleft_: Point::new(left_, top_), size_: Size::new(width_, height_).ok_or((width_, height_))?})
    }
}

impl ShrAssign<Displacement> for Rect {
    fn shr_assign(&mut self, rhs: Displacement) {
        self.topleft_ -= rhs;
    }
}

impl ShlAssign<Displacement> for Rect {
    fn shl_assign(&mut self, rhs: Displacement) {
        self.topleft_ += rhs;
    }
}

impl Shr<Displacement> for Rect {
    type Output = Self;
    fn shr(mut self, rhs: Displacement) -> Self::Output { self >>= rhs; self }
}

impl Shl<Displacement> for Rect {
    type Output = Self;
    fn shl(mut self, rhs: Displacement) -> Self::Output { self <<= rhs; self }
}

impl Mul<(Span, Rise)> for Rect {
    type Output = Point;
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
        Self::Output::new(x_, y_)
    }
}

impl Mul<(Rise, Span)> for Rect {
    type Output = Point;
    fn mul(self, (v, h): (Rise, Span)) -> Self::Output {
        self * (h, v)
    }
}

impl Div<(Span, Rise)> for Size {
    type Output = Rect;
    fn div(self, (h, v): (Span, Rise)) -> Self::Output {
        let left_ = match h {
            Span::Left => 0, Span::Center => self.width() >> 1, Span::Right => self.width(),
        } as i16;
        let top_ = match v {
            Rise::Top => 0, Rise::Center => self.height() >> 1, Rise::Bottom => self.height(),
        } as i16;
        Rect{
            topleft_: Point::new(-left_, -top_),
            size_: self,
        }
    }
}
