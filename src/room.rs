use std::slice::SliceIndex;

use super::{*, object::Object};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct RoomId(pub u16);

impl std::convert::From<RoomId> for Option<u16> {
    fn from(value: RoomId) -> Self { Some(value.0) }
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
    animate_kind: Option<Enemy>,
    animate_number: u16,
    animate_delay: Duration,
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
        step(self.left_open)..=(step(self.right_open) + 1)
    }

    pub fn theme_index(&self) -> u16 { self.back_pict_id }
}