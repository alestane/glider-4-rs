use super::{SystemTime, Success, room::Room};

#[derive(Debug)]
pub struct House {
    pub version: u16,
    pub time_stamp: SystemTime,
    pub hi_scores: Vec<Success>,
    pub pict_file: String,
    pub next_file: String,
    pub first_file: String,
    pub rooms: Vec<Room>,
}