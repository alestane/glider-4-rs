use std::ops::Range;
use glider::Room;
use crate::resources;

pub fn new() -> Room {
    Room::try_from((0, &resources::THE_HOUSE[1270..1584][..76])).unwrap()
}

const HOUSE_HEADER: usize = 1270;
const ROOM_SIZE: usize = 314;

fn index(i: usize) -> Range<usize> {
    let start = HOUSE_HEADER + i * ROOM_SIZE;
    start..(start + ROOM_SIZE)
}

const fn limit(i: usize) -> usize { 58 + i * 16 }

pub fn house() -> Box<[Room]> {
    Box::new([
        Room::try_from((1, &resources::THE_HOUSE[index(0)])).unwrap(),
        Room::try_from((2, &resources::THE_HOUSE[index(1)])).unwrap(),
        Room::try_from((3, &resources::THE_HOUSE[index(2)])).unwrap(),
        Room::try_from((4, &resources::THE_HOUSE[index(3)])).unwrap(),
        Room::try_from((5, &resources::THE_HOUSE[index(4)])).unwrap(),
        Room::try_from((6, &resources::THE_HOUSE[index(5)])).unwrap(),
        Room::try_from((7, &resources::THE_HOUSE[index(6)])).unwrap(),
        Room::try_from((8, &resources::THE_HOUSE[index(7)])).unwrap(),
        Room::try_from((9, &resources::THE_HOUSE[index(8)])).unwrap(),
        Room::try_from((10, &resources::THE_HOUSE[index(9)])).unwrap(),
        Room::try_from((11, &resources::THE_HOUSE[index(10)])).unwrap(),
    ])
}
