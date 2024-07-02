use std::{
    clone::Clone, cmp::{Eq, PartialEq}, fmt::Debug, marker::Copy, 
    num::{NonZero, ZeroablePrimitive}, 
    ops::{Add, AddAssign, Sub, SubAssign, DivAssign, Div, MulAssign, Mul, ShlAssign, Shl, ShrAssign, Shr, Neg, BitAnd}
};

#[const_trait]
pub trait Transfer: Sized {
    type Unsigned: ~const Transfer<Unsigned = Self::Unsigned, Signed = Self::Signed> + TryFrom<Self::Signed>;
    type Signed: ~const Transfer<Signed = Self::Signed, Unsigned = Self::Unsigned> + TryFrom<Self::Unsigned>;
    fn as_signed(&self) -> Self::Signed;
    fn as_unsigned(&self) -> Self::Unsigned;
}

#[const_trait]
pub trait Combine: Sized + Transfer<Unsigned: Copy, Signed: Copy> + Copy {
    fn add_signed(self, rhs: Self::Signed) -> Self;
    fn add_unsigned(self, rhs: Self::Unsigned) -> Self;
    fn sub_signed(self, rhs: Self::Signed) -> Self;
    fn sub_unsigned(self, rhs: Self::Unsigned) -> Self;
    fn difference(self, rhs: Self) -> Self::Unsigned;
    fn min(lhs: Self, rhs: Self) -> Self;
    fn max(lhs: Self, rhs: Self) -> Self;
}

impl const Transfer for u16 {
    type Unsigned = u16;
    type Signed = i16;
    fn as_signed(&self) -> Self::Signed { 0i16.saturating_add_unsigned(*self) }
    fn as_unsigned(&self) -> Self::Unsigned { *self }
}

impl const Combine for u16 {
    fn add_signed(self, rhs: Self::Signed) -> Self { self.saturating_add_signed(rhs) }
    fn add_unsigned(self, rhs: Self::Unsigned) -> Self { self.saturating_add(rhs) }
    fn sub_signed(self, rhs: Self::Signed) -> Self { self.saturating_add_signed(-rhs) }
    fn sub_unsigned(self, rhs: Self::Unsigned) -> Self { self.saturating_sub(rhs) }
    fn difference(self, rhs: Self) -> Self::Unsigned { self.abs_diff(rhs) }
    fn min(lhs: Self, rhs: Self) -> Self { if lhs < rhs {lhs} else {rhs} }
    fn max(lhs: Self, rhs: Self) -> Self { if lhs > rhs {lhs} else {rhs} }
}

impl const Transfer for i16 {
    type Unsigned = u16;
    type Signed = i16;
    fn as_signed(&self) -> Self::Signed { *self }
    fn as_unsigned(&self) -> Self::Unsigned { 0u16.saturating_add_signed(*self) }
}

impl const Combine for i16 {
    fn add_signed(self, rhs: Self::Signed) -> Self { self.saturating_add(rhs) }
    fn add_unsigned(self, rhs: Self::Unsigned) -> Self { self.saturating_add_unsigned(rhs) }
    fn sub_signed(self, rhs: Self::Signed) -> Self { self.saturating_sub(rhs)}
    fn sub_unsigned(self, rhs: Self::Unsigned) -> Self { self.saturating_sub_unsigned(rhs) }
    fn difference(self, rhs: Self) -> Self::Unsigned { self.abs_diff(rhs) }
    fn min(lhs: Self, rhs: Self) -> Self { if lhs < rhs {lhs} else {rhs} }
    fn max(lhs: Self, rhs: Self) -> Self { if lhs > rhs {lhs} else {rhs} }
}

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

impl<N: Copy> Point<N> {
    pub const fn new(x: N, y: N) -> Self { Self{x_: x, y_: y} }
    pub const fn x(&self) -> N { self.x_ }
    pub const fn y(&self) -> N { self.y_ }
}

impl<T> const Transfer for Point<T> 
where 
    T: ~const Transfer,
    Point<T::Unsigned>: TryFrom<Point<T::Signed>>,
    Point<T::Signed>: TryFrom<Point<T::Unsigned>>
{
    type Unsigned = Point<T::Unsigned>;
    type Signed = Point<T::Signed>;
    fn as_signed(&self) -> Self::Signed { Self::Signed{x_: self.x_.as_signed(), y_: self.y_.as_signed()} }
    fn as_unsigned(&self) -> Self::Unsigned { Self::Unsigned{x_: self.x_.as_unsigned(), y_: self.y_.as_unsigned()} }
}

impl<T> const Combine for Point<T> 
where
    T: ~const Combine,
    Point<T::Unsigned>: TryFrom<Point<T::Signed>>,
    Point<T::Signed>: TryFrom<Point<T::Unsigned>>
{
    fn add_signed(self, rhs: Self::Signed) -> Self { Self{x_: self.x_.add_signed(rhs.x_), y_: self.y_.add_signed(rhs.y_)} }
    fn add_unsigned(self, rhs: Self::Unsigned) -> Self { Self{x_: self.x_.add_unsigned(rhs.x_), y_: self.y_.add_unsigned(rhs.y_)} }
    fn sub_signed(self, rhs: Self::Signed) -> Self { Self{x_: self.x_.sub_signed(rhs.x_), y_: self.y_.sub_signed(rhs.y_)} }
    fn sub_unsigned(self, rhs: Self::Unsigned) -> Self { Self{x_: self.x_.sub_unsigned(rhs.x_), y_: self.y_.sub_unsigned(rhs.y_)} }
    fn difference(self, rhs: Self) -> Self::Unsigned { Self::Unsigned{ x_: self.x_.difference(rhs.x_), y_: self.y_.difference(rhs.y_) } }
    fn min(lhs: Self, rhs: Self) -> Self { Self{x_: T::min(lhs.x_, rhs.x_), y_: T::min(lhs.y_, rhs.y_)} }
    fn max(lhs: Self, rhs: Self) -> Self { Self{x_: T::max(lhs.x_, rhs.x_), y_: T::max(lhs.y_, rhs.y_)} }    
}

impl TryFrom<Point<i16>> for Point<u16> {
    type Error = <u16 as TryFrom<i16>>::Error;
    fn try_from(value: Point<i16>) -> Result<Self, Self::Error> {
        Ok(Self{x_: u16::try_from(value.x_)?, y_: u16::try_from(value.y_)?})
    }
}

impl TryFrom<Point<u16>> for Point<i16> {
    type Error = <i16 as TryFrom<u16>>::Error;
    fn try_from(value: Point<u16>) -> Result<Self, Self::Error> {
        Ok(Self{ x_: i16::try_from(value.x_)?, y_: i16::try_from(value.x_)?})
    }
}

impl<N: Copy> From<N> for Point<N> {
    fn from(value: N) -> Self { Self{x_: value, y_: value} }
}

impl<N: Copy> From<(N, N)> for Point<N> {
    fn from((x_, y_): (N, N)) -> Self { Self{x_, y_} }
}

impl<N: Copy> From<Point<N>> for (N, N) {
	fn from(Point{x_, y_}: Point<N>) -> Self { (x_, y_) }
}

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Displacement<T: Transfer<Signed = T>> {
    x_: T,
    y_: T,
}

impl<T: Transfer<Signed = T>> Displacement<T> {
    pub const fn new(x_: T, y_: T) -> Self { Self{x_, y_} }
    pub const fn x(&self) -> T where T: Copy { self.x_ }
    pub const fn y(&self) -> T where T: Copy { self.y_ }
    pub fn x_ref(&self) -> &T { &self.x_ }
    pub fn y_ref(&self) -> &T { &self.y_ }
    pub fn x_mut(&mut self) -> &mut T { &mut self.x_ }
    pub fn y_mut(&mut self) -> &mut T { &mut self.y_ }
    
    pub fn as_ref(&self) -> (&T, &T) { (&self.x_, &self.y_) }
    pub fn as_mut(&mut self) -> (&mut T, &mut T) { (&mut self.x_, &mut self.y_) }
}

impl<T> Neg for Displacement<T> 
where
    T: Transfer<Signed = T> + Neg<Output = T>
{
    type Output = Self;
    fn neg(self) -> Self::Output { Self{x_: -self.x_, y_: -self.y_} }
}

impl<T: Transfer<Signed = T> + From<i8>> Default for Displacement<T> {
    fn default() -> Self { Self{x_: 0i8.into(), y_: 0i8.into()} }
}

impl<T: Transfer<Signed = T> + Copy> From<(T, T)> for Displacement<T> {
    fn from((x_, y_): (T, T)) -> Self { Self{x_, y_} }
}

impl<T: Transfer<Signed = T> + Copy> From<Displacement<T>> for (T, T) {
    fn from(value: Displacement<T>) -> Self { (value.x_, value.y_) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size<T: Transfer<Unsigned = T> + ZeroablePrimitive>{
    width_: NonZero<T::Unsigned>,
    height_: NonZero<T::Unsigned>,
}

impl<T: Transfer<Unsigned = T> + ZeroablePrimitive> Size<T> {
    pub(crate) const fn new(width: T, height: T) -> Option<Self> { 
        match (NonZero::new(width), NonZero::new(height)) {
            (Some(width_), Some(height_)) => Some(Self{width_, height_}),
            _ => None,
        } 
    }
    pub(crate) const unsafe fn new_unchecked(width_: T, height_: T) -> Self { 
        let (width_, height_) =  (NonZero::new_unchecked(width_), NonZero::new_unchecked(height_));
        Self { width_, height_}
    }
    pub const fn width(&self) -> T { self.width_.get() }
    pub const fn height(&self) -> T { self.height_.get() }
}

impl<T: Transfer<Unsigned = T> + ZeroablePrimitive> From<(NonZero<T>, NonZero<T>)> for Size<T> {
    fn from(value: (NonZero<T>, NonZero<T>)) -> Self { Self{width_: value.0, height_: value.1} }
}

pub(crate) enum Span {Left = -1, Center = 0, Right = 1}
pub(crate) enum Rise {Top = -1, Center = 0, Bottom = 1}

#[repr(C)]
#[derive(Debug)]
pub struct Rect<T> 
where 
    T: Debug + Combine<Unsigned: ZeroablePrimitive>
{
    left_: T,
    top_: T,
    width_: NonZero<T::Unsigned>,
    height_: NonZero<T::Unsigned>,
}

impl<T> Clone for Rect<T> 
where 
    T: Combine<Unsigned: ZeroablePrimitive> + Debug
{
    fn clone(&self) -> Self { *self }
}

impl<T> Copy for Rect<T> 
where 
    T: Combine<Unsigned: Copy + ZeroablePrimitive> + Debug + Copy
{}

impl<T> PartialEq for Rect<T> 
where
    T: Combine<Unsigned: ZeroablePrimitive + PartialEq> + Debug + PartialEq
{
    fn eq(&self, other: &Self) -> bool { 
        self.left_ == other.left_ && self.top_ == other.top_ && self.width_ == other.width_ && self.height_ == other.height_
    }
}

impl<T> Eq for Rect<T> 
where
    T: Combine<Unsigned: ZeroablePrimitive + PartialEq> + Debug + PartialEq
{}

impl <T> Rect<T> 
where
    T: Debug + Combine<Unsigned: ZeroablePrimitive>
{
    pub const fn new(left: T, top: T, right: T, bottom: T) -> Option<Self> where T: ~const Combine {
        let (left_, top_) = (<T as Combine>::min(left, right), <T as Combine>::min(top, bottom));
        let (Some(width_), Some(height_)) = (NonZero::new(right.difference(left)), NonZero::new(bottom.difference(top))) else {
            return None
        };
        Some(Self {
            left_, top_, width_, height_,
        })
    }

    pub(crate) const unsafe fn new_unchecked(left: T, top: T, right: T, bottom: T) -> Self where T: ~const Combine {
        let (left_, top_) = (<T as Combine>::min(left, right), <T as Combine>::min(top, bottom));
        let width_ = NonZero::new_unchecked(right.difference(left));
        let height_ = NonZero::new_unchecked(bottom.difference(top));
        Self{left_, top_, width_, height_}
    }

    pub const fn left  (&self) -> T { self.left_   }
    pub const fn top   (&self) -> T { self.top_    }
    pub const fn right (&self) -> T where T: ~const Combine { self.left_.add_unsigned(self.width_.get()) }
    pub const fn bottom(&self) -> T where T: ~const Combine { self.top_.add_unsigned(self.height_.get()) }

    pub const fn width (&self) -> NonZero<T::Unsigned> { self.width_ }
    pub const fn height(&self) -> NonZero<T::Unsigned> { self.height_ }

    pub const fn size(&self) -> Size<T::Unsigned> { Size{width_: self.width_, height_: self.height_} }
}

impl <T> Rect<T> 
where
    T: Combine<Unsigned: ZeroablePrimitive + Transfer<Signed = T::Signed> + From<u8> + Div<Output = T::Unsigned>>
     + Debug
{
    pub(crate) fn cropped_on(center: (T, T), width: NonZero<T::Unsigned>, height: NonZero<T::Unsigned>) -> Option<Self> {
        let (width_, height_) = (width.get(), height.get());
        Some(Self{
            left_: center.0.sub_unsigned(width_ / T::Unsigned::from(2)),
            top_: center.1.sub_unsigned(height_ / T::Unsigned::from(2)),
            width_: NonZero::new(center.0.add_unsigned(width_).difference(center.0))?,
            height_: NonZero::new(center.1.add_unsigned(height_).difference(center.1))?,
        })
    }

    pub(crate) const fn clamped_on(center: (T, T), width: NonZero<T::Unsigned>, height: NonZero<T::Unsigned>) -> Self where T: ~const Combine {
        let (width_, height_) = (width, height);
        let (width, height) = (width_.get(), height_.get());
        let left_ = center.0.sub_unsigned(width / 2.into());
        let top_ = center.1.sub_unsigned(height / 2.into());
        let right_ = left_.add_unsigned(width);
        let bottom_ = top_.add_unsigned(height);
        let (left, top) = (right_.sub_unsigned(width), bottom_.sub_unsigned(height));
        let left_ = <T as Combine>::min(left_, left);
        let top_ = <T as Combine>::min(top_, top);
        Self {
            left_, top_, width_, height_
        }
    }

    pub const fn x(&self) -> T where T: ~const Combine { self.left_.add_unsigned(self.width_.get() / 2.into()) }
    pub const fn y(&self) -> T where T: ~const Combine { self.top_.add_unsigned(self.height_.get() / 2.into()) }

    pub const fn center(&self) -> Point<T> where T: ~const Combine { Point::new( self.x(), self.y() ) }
}

impl Default for Rect<u16> {
    fn default() -> Self {
        unsafe { Rect{left_: 0, top_: 0, width_: NonZero::new_unchecked(1), height_: NonZero::new_unchecked(1)} }
    }
}

impl Default for Rect<i16> {
    fn default() -> Self {
        unsafe { Rect{left_: 0, top_: 0, width_: NonZero::new_unchecked(1), height_: NonZero::new_unchecked(1)} }
    }
}

impl<T> Transfer for Rect<T> 
where 
    T: Debug + Combine<
        Unsigned: ZeroablePrimitive + Debug + Combine<Unsigned = T::Unsigned, Signed = T::Signed>, 
        Signed: Debug + Combine<Signed = T::Signed, Unsigned = T::Unsigned>
    >,
    Rect<T::Signed>: TryFrom<Rect<T::Unsigned>>,
    Rect<T::Unsigned>: TryFrom<Rect<T::Signed>>
{
    type Signed = Rect<T::Signed>;
    type Unsigned = Rect<T::Unsigned>;
    fn as_signed(&self) -> Self::Signed { 
        Self::Signed{left_: self.left_.as_signed(), top_: self.top_.as_signed(), width_: self.width_, height_: self.height_} 
    }
    fn as_unsigned(&self) -> Self::Unsigned {
        Self::Unsigned {
            left_: self.left_.as_unsigned(), top_: self.top_.as_unsigned(),
            width_: self.width_, height_: self.height_
        }
    }
}

impl TryFrom<Rect<u16>> for Rect<i16> {
    type Error = <i16 as TryFrom<u16>>::Error;
    fn try_from(value: Rect<u16>) -> Result<Self, Self::Error> {
        let left = i16::try_from(value.left_)?;
        let top = i16::try_from(value.top_)?;
        Ok(unsafe{ Self::new_unchecked(
            left,
            top,
            left.saturating_add_unsigned(value.width_.get()),
            top.saturating_add_unsigned(value.height_.get())
        )})
    }
}

impl TryFrom<Rect<i16>> for Rect<u16> {
    type Error = <u16 as TryFrom<i16>>::Error;
    fn try_from(value: Rect<i16>) -> Result<Self, Self::Error> {
        let right = u16::try_from(value.left_.saturating_add_unsigned(value.width_.get()) - 1)? + 1;
        let bottom = u16::try_from(value.top_.saturating_add_unsigned(value.height_.get()) - 1)? + 1;
        Ok(unsafe{ Self::new_unchecked(
            0u16.saturating_add_signed(value.left()),
            0u16.saturating_add_signed(value.top()),
            right,
            bottom
        ) })
    }
}

impl<T> BitAnd for Rect<T> 
where
    T: Debug + PartialOrd
     + Combine<Unsigned: ZeroablePrimitive + Div<Output = T::Unsigned> + From<u8>>
{
    type Output = Option<Self>;
    fn bitand(self, rhs: Self) -> Self::Output {
        let left_ = <T as Combine>::max(self.left_, rhs.left_);
        let right_ = <T as Combine>::min(self.right(), rhs.right());
        let top_ = <T as Combine>::max(self.top_, rhs.top_);
        let bottom_ = <T as Combine>::min(self.bottom(), rhs.bottom());
        ((left_ < right_) & (top_ < bottom_)).then_some(unsafe{Self{left_, top_, width_: NonZero::new_unchecked(right_.difference(left_)), height_: NonZero::new_unchecked(bottom_.difference(top_))}})
    }
}

impl<T> BitAnd<Option<Rect<T>>> for Rect<T> 
where
    T: Debug + PartialOrd
     + Combine<Unsigned: ZeroablePrimitive + Div<Output = T::Unsigned> + From<u8>>
{
	type Output = Option<Rect<T>>;
	fn bitand(self, rhs: Option<Self>) -> Self::Output { rhs.and_then(|b| self & b) }
}

impl<T> BitAnd<Rect<T>> for Option<Rect<T>> 
where
    T: Debug + PartialOrd
     + Combine<Unsigned: ZeroablePrimitive + Div<Output = T::Unsigned> + From<u8>>
{
	type Output = Option<Rect<T>>;
	fn bitand(self, rhs: Rect<T>) -> Self::Output { self.and_then(|a| a & rhs) }
}

impl<T> From<Rect<T>> for (T, T, T, T) 
where
T: Debug + Combine<Unsigned: ZeroablePrimitive>
{
    fn from(value: Rect<T>) -> Self {
        (value.left_, value.top_, value.right(), value.bottom())
    }
}

impl<T> From<(T, T, NonZero<T::Unsigned>, NonZero<T::Unsigned>)> for Rect<T> 
where 
    T: Debug + Combine<Unsigned: ZeroablePrimitive>
{
    fn from((left_, top_, width_, height_): (T, T, NonZero<T::Unsigned>, NonZero<T::Unsigned>)) -> Self {
        Self{left_, top_, width_, height_}
    }
}

impl<T> TryFrom<(T, T, T, T)> for Rect<T> 
where
    T: Debug + Combine<Unsigned: ZeroablePrimitive>
{
    type Error = (T::Unsigned, T::Unsigned);
    fn try_from(value: (T, T, T, T)) -> Result<Self, Self::Error> {
        Self::new(value.0, value.1, value.2, value.3).ok_or((value.2.difference(value.0), value.3.difference(value.1)))
    }
}

impl<T, U> ShrAssign<Point<U>> for Rect<T> 
where
    T: Debug + Combine<Unsigned: ZeroablePrimitive>,
    U: Debug + Combine<Unsigned: ZeroablePrimitive>, 
    T: Combine<Signed = U::Signed, Unsigned = U::Unsigned>
{
    fn shr_assign(&mut self, rhs: Point<U>) {
        let (width, height) = (self.width_.get(), self.height_.get());
        self.left_ = <T as Combine>::min(self.left_.add_signed(rhs.x_.as_signed()), self.left_.add_unsigned(width).sub_unsigned(width));
        self.top_ = <T as Combine>::min(self.top_.add_signed(rhs.y_.as_signed()), self.top_.add_unsigned(height).sub_unsigned(height));
    }
}

impl<T, I> ShrAssign<I> for Rect<T>
where 
    T: Debug + Combine<Unsigned: ZeroablePrimitive>,
    I: Into<Displacement<T::Signed>>, 
    Self: ShrAssign<Point<T::Signed>>
{
    fn shr_assign(&mut self, rhs: I) {
        let Displacement{x_, y_} = rhs.into();
        *self >>= Point::<T::Signed>::new(x_, y_)
    }
}

impl<T, U> ShlAssign<Point<U>> for Rect<T> 
where
    T: Debug + Combine<Unsigned: ZeroablePrimitive>,
    U: Debug + Combine<Unsigned: ZeroablePrimitive>, 
    T: Combine<Signed = U::Signed, Unsigned = U::Unsigned>
{
    fn shl_assign(&mut self, rhs: Point<U>) {
        let (width, height) = (self.width_.get(), self.height_.get());
        self.left_ = <T as Combine>::min(self.left_.sub_signed(rhs.x_.as_signed()), self.left_.add_unsigned(width).sub_unsigned(width));
        self.top_ = <T as Combine>::min(self.top_.sub_signed(rhs.y_.as_signed()), self.top_.add_unsigned(height).sub_unsigned(height));
    }
}

impl<T, I> ShlAssign<I> for Rect<T>
where 
    T: Debug + Combine<Unsigned: ZeroablePrimitive, Signed: Neg<Output = T::Signed>>,
    I: Into<Displacement<T::Signed>>, 
    Self: ShlAssign<Point<T::Signed>>
{
    fn shl_assign(&mut self, rhs: I) {
        let Displacement{x_, y_} = -rhs.into();
        *self <<= Point::<T::Signed>::new(x_, y_)
    }
}

impl<T, U> Shr<Point<U>> for Rect<T> 
where
    T: Debug + Combine<Unsigned: ZeroablePrimitive>,
    U: Debug + Combine<Unsigned: ZeroablePrimitive>, 
    T: Combine<Signed = U::Signed, Unsigned = U::Unsigned>,
    Self: ShrAssign<Point<U>>
{
    type Output = Self;
    fn shr(mut self, rhs: Point<U>) -> Self::Output { self >>= rhs; self }
}

impl<T, U> Shl<Point<U>> for Rect<T> 
where
    T: Debug + Combine<Unsigned: ZeroablePrimitive>,
    U: Debug + Combine<Unsigned: ZeroablePrimitive>, 
    T: Combine<Signed = U::Signed, Unsigned = U::Unsigned>,
    Self: ShlAssign<Point<U>>
{
    type Output = Self;
    fn shl(mut self, rhs: Point<U>) -> Self::Output { self >>= rhs; self }
}

impl<T> Mul<(Span, Rise)> for Rect<T> 
where 
    T: Debug + Combine<Unsigned: ZeroablePrimitive + Div<Output = T::Unsigned> + From<u8>>,
    T::Signed: Transfer<Unsigned = T::Unsigned>
{
    type Output = Point<T>;
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

impl<T> Mul<(Rise, Span)> for Rect<T> 
where 
    T: Debug + Combine<Unsigned: ZeroablePrimitive + Div<Output = T::Unsigned> + From<u8>>,
    T::Signed: Transfer<Unsigned = T::Unsigned>
{
    type Output = Point<T>;
    fn mul(self, (v, h): (Rise, Span)) -> Self::Output {
        self * (h, v)
    }
}

impl<T> Div<(Span, Rise)> for Size<T>
where 
    T: ZeroablePrimitive + Combine<Unsigned = T> + Div<Output = T::Unsigned> + From<u8>,
    T::Signed: Debug + Eq + Combine<Unsigned = T> + From<i8>,
    <T::Signed as Transfer>::Unsigned: ZeroablePrimitive 
{
    type Output = Rect<T::Signed>;
    fn div(self, (h, v): (Span, Rise)) -> Self::Output {
        let left_ = T::Signed::from(0i8).sub_unsigned( match h {
            Span::Left => 0.into(), Span::Center => self.width() / 2u8.into(), Span::Right => self.width(),
        } );
        let top_ = T::Signed::from(0i8).sub_unsigned( match v {
            Rise::Top => 0.into(), Rise::Center => self.height() / 2u8.into(), Rise::Bottom => self.height()
        } );
        let (width_, height_) = (self.width_, self.height_);
        Rect{
            left_,
            top_,
            width_,
            height_,
        }
    }
}
