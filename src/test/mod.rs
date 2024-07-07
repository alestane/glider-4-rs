use std::{num::NonZero, ops::Range};
use glider::Room;
use crate::resources;

pub fn new() -> Room {
    Room::try_from((const{NonZero::new(1u16).unwrap()}, &resources::THE_HOUSE[1270..1584][..76])).unwrap()
}

const HOUSE_HEADER: usize = 1270;
const ROOM_SIZE: usize = 314;

pub const START: NonZero<u16> = NonZero::new(13).unwrap();

fn index(i: usize) -> Range<usize> {
    let start = HOUSE_HEADER + i * ROOM_SIZE;
    start..(start + ROOM_SIZE)
}

#[allow(dead_code)]
const fn limit(i: usize) -> usize { 58 + i * 16 }

pub fn house() -> Box<[Room]> {
    let mut zip = 1u16..;
    Box::new(
        [0; 13].map(move |_| zip.next().and_then(NonZero::new).unwrap()).map(|id|
            Room::try_from((id, &resources::THE_HOUSE[index(id.get() as usize - 1)])).unwrap()
        )
    )
}
