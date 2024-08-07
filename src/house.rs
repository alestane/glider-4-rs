use crate::object;

use super::{Success, room::{self, Room}, prelude::Travel};
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
        type Kind = object::Kind;
        let offset = self.rooms.len() as u16;
        for room in &mut rhs.rooms {
            if let Some(left) = &mut room.exits.left { *left += offset; }
            if let Some(right) = &mut room.exits.right { *right += offset; }
            for object in room {
                match object.kind {
                    Kind::CeilingDuct(Travel(Some(ref mut destination))) |
                    Kind::Exit { to: Some(ref mut destination), .. } |
                    Kind::Stair(_, ref mut destination)
                        => *destination += offset,
                    _ => ()
                }
            }
        }
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