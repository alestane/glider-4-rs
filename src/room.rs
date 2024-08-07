use std::{num::NonZero, slice::SliceIndex, ops::{AddAssign, Index}};

use super::{*, object::Object};

pub const SCREEN_HEIGHT:	i16 = 342;
pub const SCREEN_WIDTH:		i16 = 512;
pub const VERT_CEILING:		i16 = 24;
pub const VERT_FLOOR:		i16 = 325;

pub const BOUNDS:	Bounds = const{ Bounds::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap() };

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Id(pub(crate) NonZero<u16>);

impl From<u16> for self::Id {
	fn from(value: u16) -> Self { unsafe { Self( NonZero::new_unchecked( value.saturating_sub(1) + 1 ) ) } }
}

impl From<usize> for self::Id {
	fn from(value: usize) -> Self { unsafe { Self( NonZero::new_unchecked((value + 1) as u16) ) } }
}

impl From<self::Id> for usize {
	fn from(value: Id) -> Self { value.0.get() as usize - 1 }
}

impl From<self::Id> for Option<u16> {
    fn from(value: Id) -> Self { Some(value.0.get()) }
}

impl Id {
    pub fn prev(&self) -> Option<Id> { Some(Id(NonZero::new(self.0.get() - 1)?)) }
    pub fn next(&self) -> Option<Id> { Some(Id(self.0.checked_add(1)?)) }
}

impl AddAssign<u16> for Id {
    fn add_assign(&mut self, rhs: u16) {
        self.0 = self.0.saturating_add(rhs);
    }
}

#[disclose]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct On {
    air: bool,
    lights: bool,
}

#[disclose]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Exits {
    left: Option<Id>,
    right: Option<Id>,
}

impl Index<Side> for Exits {
    type Output = Option<Id>;
    fn index(&self, index: Side) -> &Self::Output {
        match index {
            Side::Left => &self.left,
            Side::Right => &self.right,
        }
    }
}

impl Exits {
    pub fn walls(&self) -> impl SliceIndex<[Object], Output=[Object]> {
        fn step(i: Option<room::Id>) -> usize { i.is_some() as usize }

        step(self.left)..=(2 - step(self.right))
    }
}

#[disclose]
#[derive(Debug)]
pub struct Room {
    name: String,
    back_pict_id: u16,
    tile_order: [u8; 8],
    exits: Exits,
    animate: Option<(NonZero<u16>, object::Kind)>,
    environs: On,
    objects: Vec<Object>,
}

impl Index<object::Id> for Room {
    type Output = Object;
    fn index(&self, index: object::Id) -> &Self::Output {
        &self.objects[usize::from(index)]
    }
}

#[derive(Debug)]
pub enum RoomImportError<'a> {
    ShortData(&'a [u8]),
    TranscriptionErr(<Room as TryFrom<(Id, &'a [u8])>>::Error),
}

impl<'a> From<<Room as TryFrom<(Id, &'a [u8])>>::Error> for RoomImportError<'a> {
    fn from(value: <Room as TryFrom<(Id, &'a [u8])>>::Error) -> Self { Self::TranscriptionErr(value) }
}

impl<'a> TryFrom<(NonZero<u16>, &'a [u8])> for Room {
    type Error = RoomImportError<'a>;
    fn try_from((id, data): (NonZero<u16>, &'a [u8])) -> Result<Self, Self::Error> {
        if data.len() < import::ROOM_SIZE {
            Err( RoomImportError::ShortData(data) )
        } else {
            Ok(Self::try_from((Id(id), &data[..import::ROOM_SIZE]))?)
        }
    }
}

impl Room {
    pub fn walls(&self) -> impl SliceIndex<[Object], Output=[Object]> { self.exits.walls() }

    pub fn len(&self) -> usize { self.objects.len() }
    
    pub fn theme_index(&self) -> u16 { self.back_pict_id }
}
        
impl std::ops::Index<Side> for Room {
	type Output = Option<room::Id>;
	fn index(&self, which: Side) -> &Self::Output { &self.exits[which] }
}

impl IntoIterator for Room {
    type Item = Object;
    type IntoIter = <Vec<Object> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter { self.objects.into_iter() }
}

impl<'a> IntoIterator for &'a Room {
    type Item = &'a Object;
    type IntoIter = <&'a Vec<Object> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter { (&self.objects).into_iter() }
}

impl<'a> IntoIterator for &'a mut Room {
    type Item = &'a mut Object;
    type IntoIter = <&'a mut Vec<Object> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter { (&mut self.objects).into_iter() }
}