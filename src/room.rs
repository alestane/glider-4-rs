use std::{num::NonZero, slice::SliceIndex};

use super::{*, object::Object};

pub const SCREEN_HEIGHT:	u16 = 342;
pub const SCREEN_WIDTH:		u16 = 512;
pub const VERT_CEILING:		u16 = 24;
pub const VERT_FLOOR:		u16 = 325;

pub const BOUNDS:	Bounds = unsafe { Rect::new_unchecked(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT) };

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
/* 
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

impl From<ObjectKind> for Option<Enemy> {
    fn from(value: ObjectKind) -> Self {
        Some(match value {
            ObjectKind::Candle { .. } => Enemy::Flame,
            ObjectKind::Fishbowl { .. } => Enemy::Fish,
            ObjectKind::Ball{ .. } => Enemy::Ball,
            ObjectKind::Toaster { .. } => Enemy::Toast,
            ObjectKind::Outlet { .. } => Enemy::Shock,
            _ => return None
        })
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Deactivated {
    Air = 1,
    Lights = 2,
}

#[disclose]
#[derive(Debug)]
pub struct Room {
    name: String,
    back_pict_id: u16,
    tile_order: [u16; 8],
    left_open: Option<RoomId>,
    right_open: Option<RoomId>,
    animate: Option<(Enemy, NonZero<u16>, u32)>,
    condition_code: Option<Deactivated>,
    objects: Vec<Object>,
}

impl TryFrom<(u16, &[u8])> for Room {
    type Error = ();
    fn try_from(data: (u16, &[u8])) -> Result<Self, Self::Error> {
        if data.1.len() < 58 {
            Err( Self::Error::default() )
        } else {
            Ok(Self::from((data.0, import::RoomData::from_iter(data.1.iter().copied()))))
        }
    }
}

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