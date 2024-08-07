use std::{num::NonZero, ops::Range};
use glider::Room;
use crate::resources;

const HOUSE_HEADER: usize = 1270;
const ROOM_SIZE: usize = 314;

pub const START: NonZero<u16> = NonZero::new(40).unwrap();

fn index(i: usize) -> Range<usize> {
    let start = HOUSE_HEADER + i * ROOM_SIZE;
    start..(start + ROOM_SIZE)
}

#[allow(dead_code)]
const fn limit(i: usize) -> usize { 58 + i * 16 }

pub fn house() -> Box<[Room]> {
    let mut zip = 1u16..;
    Box::new(
        [0; 40].map(move |_| zip.next().and_then(NonZero::new).unwrap()).map(|id|
            Room::try_from((id, &resources::THE_HOUSE[index(id.get() as usize - 1)])).unwrap()
        )
    )
}

pub fn stock_house() -> glider::House {
    let mut a = resources::THE_HOUSE.try_into().unwrap();
    a += resources::TWO_HOUSE.try_into().unwrap();
    a
}