use glider::Room;

use crate::resources;

pub fn new() -> Room {
    Room::try_from((0, &resources::THE_HOUSE[1270..1584])).unwrap()
}
