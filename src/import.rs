use super::{*, room::*, object::*, house::*};

fn string_from_pascal(bytes: &[u8]) -> String {
    String::from_utf8_lossy(match bytes {
        [len, chars@..] if *len as usize <= chars.len() => &chars[..*len as usize],
        [_, chars@..] => chars, 
        _ => return String::new()
    }).to_string()
}

impl ObjectKind {
    pub fn try_from_raw(kind: u16, amount: u16, extra: u16) -> Result<Self, Option<()>> {
        use ObjectKind::*;

        Ok(match kind { 
             0 => return Err(None),

             1 => Table,
             2 => Shelf,
             3 => Books,
             4 => Cabinet,
             5 => Exit{to: RoomId(amount.into())},
             6 => Obstacle,

             8 => FloorVent{height:amount},
             9 => CeilingVent{height: amount},
            10 => CeilingDuct{height: amount, destination: RoomId(extra.into())},
            11 => Candle{height: amount},
            12 => Fan{faces: Side::Left, range: amount},
            13 => Fan{faces: Side::Right, range: amount},

            16 => Clock(amount),
            17 => Paper(amount),
            18 => Grease{range: amount},
            19 => Bonus(amount),
            20 => Battery(amount),
            21 => RubberBands(amount),

            24 => Switch(None),
            25 => Outlet{delay: Duration::from_secs(amount.into()) / 30},
            26 => Thermostat,
            27 => Shredder,
            28 => Switch(Some(ObjectId(amount.into()))),
            29 => Guitar,

            32 => Drip{range: amount},
            33 => Toaster{range: amount, delay: extra},
            34 => Ball{range: amount},
            35 => Fishbowl{range: amount, delay: extra},
            36 => Teakettle{range: amount},
            37 => Window,

            40 => Painting, 
            41 => Mirror,
            42 => Basket,
            43 => Macintosh,
            44 => Stair(Vertical::Up, RoomId(amount.into())),
            45 => Stair(Vertical::Down, RoomId(amount.into())),

            _ => return Err(Some( () ) ) 
        })
    }

}

impl From<[u16; 4]> for Rect {
    fn from(data: [u16; 4]) -> Self {
        (data[1], data[0], data[3], data[2]).into()
    }
}

impl From<ObjectData> for Option<Object> {
    fn from(value: ObjectData) -> Self {
        ObjectKind::try_from_raw(u16::from_be_bytes(value.object_is), u16::from_be_bytes(value.amount), u16::from_be_bytes(value.extra))
        .ok().map(|kind|
            Object {
                object_is: kind,
                bounds: value.bounds.map(|mem| u16::from_be_bytes(mem)).into(),
                is_on: value.is_on[0] != 0,
            }
        )
    }
}

impl TryFrom<u16> for Deactivated {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::Air,
            2 => Self::Lights,
            _ => return Err(())
        })
    }
}

impl TryFrom<u16> for Enemy {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Dart,
            1 => Self::Copter,
            2 => Self::Balloon,
            _ => return Err(())
        })
    }
}

impl From<(u16, RoomData)> for Room {
    fn from((id, value): (u16, RoomData)) -> Self {
        let n_objects = u16::from_be_bytes(value.object_count) as usize;
        Self {
            name: string_from_pascal(&value.name),
            back_pict_id: u16::from_be_bytes(value.back_pict_id),
            tile_order: value.tile_order.into_iter().map(|pair| u16::from_be_bytes(pair)).next_chunk().unwrap(),
            left_open: (value.left_right_open[0] != 0).then_some(RoomId(id.saturating_sub(1))),
            right_open: (value.left_right_open[1] != 0).then_some(RoomId(id.saturating_add(1))),
            animate_kind: Enemy::try_from(u16::from_be_bytes(value.animate_kind)).ok(),
            animate_number: u16::from_be_bytes(value.animate_number),
            animate_delay: Duration::from_secs(u32::from_be_bytes(value.animate_delay) as u64) / 30,
            condition_code: Deactivated::try_from(u16::from_be_bytes(value.condition_code)).ok(),
            objects: Vec::from_iter(
                value.objects[..n_objects]
                .iter()
                .map_while(|&o| o.into() )
            ),
        }
    }
}

impl From<HouseData> for House {
    fn from(value: HouseData) -> Self {
        let n_rooms = u16::from_be_bytes(value.n_rooms) as usize;
        Self {
            version: u16::from_be_bytes(value.version),
            time_stamp: SystemTime::UNIX_EPOCH + Duration::from_secs(u32::from_be_bytes(value.time_stamp) as u64),
            hi_scores: Vec::from_iter(
                value.hi_scores.iter()
                    .zip(value.hi_level)
                    .zip(value.hi_names.iter().zip(value.hi_rooms.iter()))
                    .map(|((score, level), (name, room))|
                        Success{
                            score: u32::from_be_bytes(*score), 
                            level: u16::from_be_bytes(level),
                            name: string_from_pascal(name),
                            room: string_from_pascal(room),
                        }
                    )
                ),
            pict_file: string_from_pascal(&value.pict_name),
            next_file: string_from_pascal(&value.next_file),
            first_file: string_from_pascal(&value.first_file), 
            rooms: Vec::from_iter(value.rooms[..n_rooms].iter().enumerate().map(|(i, r)| Room::from((i as u16, *r))))
        }
    }
}

fn take_partition<I: IntoIterator, const PITCH: usize, const SIZE: usize>(i: I) -> [[I::Item; PITCH]; SIZE] where I::Item: core::fmt::Debug + Copy, [(); PITCH * SIZE]: {
    let contents = i.into_iter()
        .next_chunk::<{PITCH * SIZE}>().unwrap();
    contents
        .as_chunks::<PITCH>().0[0..SIZE].try_into().unwrap()
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ObjectData {
    object_is: [u8; 2], // 2
    bounds: [[u8; 2]; 4], // 8 // 10
    amount: [u8; 2], // 2 // 12
    extra: [u8; 2], // 2 // 14
    is_on: [u8; 2], // 1 // 16
}

impl Default for ObjectData {
    fn default() -> Self {
        Self {
            object_is: [0;2],
            bounds: [[0;2];4],
            amount: [0;2],
            extra: [0;2],
            is_on: [0;2],
        }
    }
}

impl FromIterator<u8> for ObjectData {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        Self {
            object_is: iter.next_chunk().unwrap(),
            bounds: take_partition(&mut iter),
            // iter.next_chunk::<8>().unwrap().as_chunks().0[0..4].try_into().unwrap(),
            amount: iter.next_chunk().unwrap(),
            extra: iter.next_chunk().unwrap(),
            is_on: iter.next_chunk().unwrap()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub (crate) struct RoomData {
    name: [u8; 26], // 26
    object_count: [u8; 2], // 2 // 28
    back_pict_id: [u8; 2], // 2 // 30
    tile_order: [[u8; 2]; 8], // 16 // 46
    left_right_open: [u8; 2], // 1 // 48
    animate_kind: [u8; 2], // 2 // 50
    animate_number: [u8; 2], // 2 // 52
    animate_delay: [u8; 4], // 4 // 56
    condition_code: [u8; 2], // 2 // 58
    objects: [ObjectData; 16], // 256 // 314
}

impl FromIterator<u8> for RoomData {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        Self {
            name: iter.next_chunk().unwrap(),
            object_count: iter.next_chunk().unwrap(),
            back_pict_id: iter.next_chunk().unwrap(),
            tile_order: take_partition(&mut iter),
            left_right_open: iter.next_chunk().unwrap(),
            animate_kind: iter.next_chunk().unwrap(),
            animate_number: iter.next_chunk().unwrap(),
            animate_delay: iter.next_chunk().unwrap(),
            condition_code: iter.next_chunk().unwrap(),
            objects: (0..16).map(|_| iter.next_chunk::<16>().map(|bytes| ObjectData::from_iter(bytes)).unwrap_or_default()).next_chunk().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub (crate) struct HouseData {
    version: [u8; 2], // 2
    n_rooms: [u8; 2], // 2 // 4
    time_stamp: [u8; 4], // 4 // 8
    hi_scores: [[u8; 4]; 20], // 80 // 88
    hi_level: [[u8; 2]; 20], // 40 // 128
    hi_names: [[u8; 26]; 20], // 520 // 648
    hi_rooms: [[u8; 26]; 20], // 520 // 1168
    pict_name: [u8; 34], // 34 // 1202
    next_file: [u8; 34], // 34 // 1236
    first_file: [u8; 34], // 34 // 1270
    rooms: [RoomData; 40], // 12560 // 13830
}

impl FromIterator<u8> for HouseData {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        Self {
            version: iter.next_chunk().unwrap(),
            n_rooms: iter.next_chunk().unwrap(),
            time_stamp: iter.next_chunk().unwrap(),
            hi_scores: take_partition(&mut iter),
            // iter.next_chunk::<80>().unwrap().as_chunks().0[0..20].try_into().unwrap(),
            hi_level: take_partition(&mut iter),
            // *iter.next_chunk::<40>().unwrap().as_chunks().0.split_array_ref().0,
            hi_names: take_partition(&mut iter),
            // *iter.next_chunk::<520>().unwrap().as_chunks().0.split_array_ref().0,
            hi_rooms: take_partition(&mut iter),
            // *iter.next_chunk::<520>().unwrap().as_chunks().0.split_array_ref().0,
            pict_name: iter.next_chunk().unwrap(),
            next_file: iter.next_chunk().unwrap(),
            first_file: iter.next_chunk().unwrap(),
            rooms: (0..40).map(|_| RoomData::from_iter(&mut iter)).next_chunk().unwrap(),
        }
    }
}
