use glider::Room;

use crate::resources;

const TEST_ROOM: &[u8] = 
    b"\x07Example\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
    \0\x01\
    \0\xC8\
    \0\x07\0\0\0\x05\0\0\0\0\0\x01\0\x02\0\x03\
    \0\x01\
    \0\0\0\0\0\0\0\0\
    \0\0\
    \0\x08\x01\x45\0\x45\x01\x52\0\x75\0\x2c\0\0\0\0\
    "; 

pub fn new() -> Room {
//    Room::try_from((0, &resources::THE_HOUSE[1270..1584][0..58])).unwrap()
    Room::try_from((0, &resources::THE_HOUSE[1270..1584][0..90])).unwrap()
}
