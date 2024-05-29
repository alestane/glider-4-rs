use super::{*, object::Object};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct RoomId(pub usize);

#[derive(Debug, Clone, Copy)]
pub enum Enemy {
    Dart,
    Copter,
    Balloon,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
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

impl TryFrom<(usize, &[u8])> for Room {
    type Error = ();
    fn try_from(data: (usize, &[u8])) -> Result<Self, Self::Error> {
        if data.1.len() < 58 {
            Err( Self::Error::default() )
        } else {
            Ok(Self::from((data.0, import::RoomData::from_iter(data.1.iter().copied()))))
        }
    }
}