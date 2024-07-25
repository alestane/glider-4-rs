use super::{Success, room::{self, Room}};
use std::{ops::{AddAssign, Deref, Index}, time::SystemTime};

#[disclose]
#[derive(Debug)]
pub struct House {
    version: u16,
    time_stamp: SystemTime,
    hi_scores: Vec<Success>,
    pict_file: String,
    next_file: String,
    first_file: String,
    rooms: Vec<Room>,
}

impl AddAssign for House {
    fn add_assign(&mut self, mut rhs: Self) {
        self.rooms.append(&mut rhs.rooms);
    }
}

impl Deref for House {
    type Target = [Room];
    fn deref(&self) -> &Self::Target { &self.rooms }
}

impl Index<room::Id> for House {
    type Output = Room;
    fn index(&self, index: room::Id) -> &Self::Output {
        &self.rooms[usize::from(index)]
    }
}