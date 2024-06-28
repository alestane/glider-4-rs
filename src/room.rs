use std::{num::NonZero, slice::SliceIndex, ops::Index};

use super::{*, object::Object};

pub const SCREEN_HEIGHT:	u16 = 342;
pub const SCREEN_WIDTH:		u16 = 512;
pub const VERT_CEILING:		u16 = 24;
pub const VERT_FLOOR:		u16 = 325;

pub const BOUNDS:	Bounds = unsafe { Bounds::new_unchecked(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT) };

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enemy {
    Dart,
    Copter,
    Balloon,
    Flame,
    Fish,
    Ball,
    Toast,
    Shock,
}


impl From<object::Kind> for Option<Enemy> {
    fn from(value: object::Kind) -> Self {
        Some(match value {
            object::Kind::Candle { .. } => Enemy::Flame,
            object::Kind::Fishbowl { .. } => Enemy::Fish,
            object::Kind::Ball{ .. } => Enemy::Ball,
            object::Kind::Toaster { .. } => Enemy::Toast,
            object::Kind::Outlet { .. } => Enemy::Shock,
            _ => return None
        })
    }
} 

#[disclose]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct On {
    air: bool,
    lights: bool,
}

#[disclose]
#[derive(Debug)]
pub struct Room {
    name: String,
    back_pict_id: u16,
    tile_order: [u8; 8],
    left_open: Option<Id>,
    right_open: Option<Id>,
    animate: Option<(Enemy, NonZero<u16>, u32)>,
    environs: On,
    objects: Vec<Object>,
}

impl Index<object::Id> for Room {
    type Output = Object;
    fn index(&self, index: object::Id) -> &Self::Output {
        &self.objects[usize::from(index)]
    }
}

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

/*
impl Room {
    pub fn walls(&self) -> impl SliceIndex<[Object], Output=[Object]> {
        fn step(i: Option<RoomId>) -> usize { i.is_some() as usize }
        step(self.left_open)..=(2 - step(self.right_open))
    }

    pub fn theme_index(&self) -> u16 { self.back_pict_id }
}

impl std::ops::Index<Side> for Room {
	type Output = Option<RoomId>;
	fn index(&self, which: Side) -> &Self::Output { match which {Side::Left=>&self.left_open, Side::Right=>&self.right_open} }
}
 */