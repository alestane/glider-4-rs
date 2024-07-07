use super::{Success, room::{self, Room}};
use std::{ops::Index, time::SystemTime};

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

impl Index<room::Id> for House {
    type Output = Room;
    fn index(&self, index: room::Id) -> &Self::Output {
        &self.rooms[usize::from(index)]
    }
}