use std::{fmt::Display, num::NonZero, time::{Duration, SystemTime}};

use super::{*,
    room::Room, 
    object::Object, 
    house::House,
    cart::{Rise, Span},
};

fn string_from_pascal(bytes: &[u8]) -> String {
    String::from_utf8_lossy(match bytes {
        [len, chars@..] if *len as usize <= chars.len() => &chars[..*len as usize],
        [_, chars@..] => chars,
        _ => return String::new()
    }).to_string()
}

pub enum BadRectError{
    Empty{width: Option<NonZero<u16>>, height: Option<NonZero<u16>>},
    Inverted,
}

impl TryFrom<[u16; 4]> for Bounds {
    type Error = BadRectError;
    fn try_from(data: [u16; 4]) -> Result<Self, Self::Error> {
        let (true, true) = (data[3] > data[1], data[2] > data[0]) else { return Err(BadRectError::Inverted)};
        match (NonZero::new(data[3] - data[1]), NonZero::new(data[2] - data[0])) {
            (Some(..), Some(..)) => Ok(unsafe{ Bounds::new_unchecked(data[1], data[0], data[3], data[2])}),
            (width, height) => Err(BadRectError::Empty{width, height}),
        }
    }
}

impl TryFrom<[[u8; 2]; 4]> for Bounds {
    type Error = <Bounds as TryFrom<[u16;4]>>::Error;
    fn try_from(value: [[u8; 2]; 4]) -> Result<Self, Self::Error> {
        value.map(u16::from_be_bytes).try_into()
    }
}

type Block<T> = [u8; size_of::<T>()];

pub const ROOM_SIZE: usize = size_of::<binary::Room>();
pub const OBJECT_SIZE: usize = size_of::<binary::Object>();
pub const HOUSE_SIZE: usize = size_of::<binary::House>();

mod binary {
    fn take_partition<I: IntoIterator, const PITCH: usize, const SIZE: usize>(i: I) -> [[I::Item; PITCH]; SIZE] where I::Item: core::fmt::Debug + Copy, [(); PITCH * SIZE]: {
        let contents = i.into_iter()
            .next_chunk::<{PITCH * SIZE}>().unwrap();
        contents
            .as_chunks::<PITCH>().0[0..SIZE].try_into().unwrap()
    }
    
    use super::Block;

    #[repr(C)]
    #[disclose(super)]
    #[derive(Debug, Clone, Copy)]
    pub(super) struct Object {
        object_is: [u8; 2], // 2
        bounds: [[u8; 2]; 4], // 8 // 10
        amount: [u8; 2], // 2 // 12
        extra: [u8; 2], // 2 // 14
        is_on: [u8; 2], // 1 // 16
    }

    impl Default for Object {
        fn default() -> Self { Self::from([0;_]) }
    }

    impl AsRef<Object> for Block<Object> {
        fn as_ref<'a>(&'a self) -> &'a Object {
            unsafe { (self as *const _ as *const Object).as_ref().unwrap_unchecked() }
        }
    }

    impl From<Block<Object>> for Object {
        fn from(value: Block<Object>) -> Self { *value.as_ref() }
    }

    impl<'a> TryFrom<&'a [u8]> for &'a Object {
        type Error = std::array::TryFromSliceError;
        fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
            Ok(<&Block<Object>>::try_from(value)?.as_ref())
        }
    }

    impl FromIterator<u8> for Object {
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

    #[disclose(super)]
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub(super) struct RoomHeader {
        name: [u8; 26], 
        object_count: [u8; 2],
        back_pict_id: [u8; 2], 
        tile_order: [[u8; 2]; 8], 
        left_right_open: [u8; 2], 
        animate_kind: [u8; 2], 
        animate_number: [u8; 2], 
        animate_delay: [u8; 4], 
        condition_code: [u8; 2], 
    }

    impl AsRef<RoomHeader> for Block<RoomHeader> {
        fn as_ref(&self) -> &RoomHeader { 
            unsafe { (self as *const _ as *const RoomHeader).as_ref().unwrap_unchecked() }
         }
    }

    impl From<Block<RoomHeader>> for RoomHeader {
        fn from(value: Block<RoomHeader>) -> Self { *value.as_ref() }
    }

    impl<'a> TryFrom<&'a [u8]> for &'a RoomHeader {
        type Error = std::array::TryFromSliceError;
        fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
            Ok(<&Block<RoomHeader>>::try_from(value)?.as_ref())
        }
    }

    impl FromIterator<u8> for RoomHeader {
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
            }
        }
    } 

    #[repr(C)]
    #[disclose(super)]
    #[derive(Debug, Clone, Copy)]
    pub(super) struct Room {
        header: RoomHeader,
        objects: [Object; 16],
    }

    impl AsRef<Room> for Block<Room> {
        fn as_ref(&self) -> &Room { 
            unsafe { (self as *const _ as *const Room).as_ref().unwrap_unchecked() }
         }
    }

    impl From<Block<Room>> for Room {
        fn from(value: Block<Room>) -> Self { *value.as_ref() }
    }

    impl<'a> TryFrom<&'a [u8]> for &'a Room {
        type Error = std::array::TryFromSliceError;
        fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
            Ok(<&Block<Room>>::try_from(value)?.as_ref())
        }
    }

    impl FromIterator<u8> for Room {
        fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
            let mut source = iter.into_iter();
            Self {
                header: RoomHeader::from_iter(&mut source),
                objects: [0;16].map(|_| Object::from_iter(&mut source)),
            }
        }
    }

    #[repr(C)]
    #[disclose(super)]
    #[derive(Debug, Clone, Copy)]
    pub(super) struct HouseHeader {
        version: [u8; 2],
		n_rooms: [u8; 2],
		time_stamp: [u8;4],
        hi_scores: [[u8; 4]; 20],
        hi_level: [[u8; 2]; 20],
        hi_names: [[u8; 26]; 20],
        hi_rooms: [[u8; 26]; 20],
        pict_name: [u8; 34],
        next_file: [u8; 34],
        first_file: [u8; 34],
    }

    impl AsRef<HouseHeader> for Block<HouseHeader> {
        fn as_ref(&self) -> &HouseHeader { 
            unsafe { (self as *const _ as *const HouseHeader).as_ref().unwrap_unchecked() }
         }
    }

    impl From<Block<HouseHeader>> for HouseHeader {
        fn from(value: Block<HouseHeader>) -> Self { *value.as_ref() }
    }

    impl<'a> TryFrom<&'a [u8]> for &'a HouseHeader {
        type Error = std::array::TryFromSliceError;
        fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
            Ok(<&Block<HouseHeader>>::try_from(value)?.as_ref())
        }
    }

    impl FromIterator<u8> for HouseHeader {
        fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
            let mut iter = iter.into_iter();
            Self {
                version: iter.next_chunk().unwrap(),
                n_rooms: iter.next_chunk().unwrap(),
                time_stamp: iter.next_chunk().unwrap(),
                hi_scores: take_partition(&mut iter),
                hi_level: take_partition(&mut iter),
                hi_names: take_partition(&mut iter),
                hi_rooms: take_partition(&mut iter),
                pict_name: iter.next_chunk().unwrap(),
                next_file: iter.next_chunk().unwrap(),
                first_file: iter.next_chunk().unwrap(),
            }
        }
    }     

    #[repr(C)]
    #[disclose(super)]
    #[derive(Debug, Clone, Copy)]
    pub(super) struct House {
        header: HouseHeader,
        rooms: [Room; 40],
    }

    impl AsRef<House> for Block<House> {
        fn as_ref(&self) -> &House { 
            unsafe { (self as *const _ as *const House).as_ref().unwrap_unchecked() }
         }
    }

    impl From<Block<House>> for House {
        fn from(value: Block<House>) -> Self { *value.as_ref() }
    }

    impl<'a> TryFrom<&'a [u8]> for &'a House {
        type Error = std::array::TryFromSliceError;
        fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
            Ok(<&Block<House>>::try_from(value)?.as_ref())
        }
    }

    impl FromIterator<u8> for House {
        fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
            let mut source = iter.into_iter();
            Self {
                header: HouseHeader::from_iter(&mut source),
                rooms: [0; 40].map(|_| Room::from_iter(&mut source)),
            }
        }
    }
}

impl object::Kind {
    pub(self) const fn display_anchor(&self) -> (Span, Rise) {
        type Is = object::Kind;
        match self {
            Is::Table{..} | Is::Shelf {..} |
            Is::Drip{..} |
            Is::FloorVent{..}
                => (Span::Center, Rise::Top),
            Is::Exit{..} |
            Is::Painting{..} | Is::Mirror(..) | Is::Window(..) |
            Is::Bonus(..) |
            Is::Switch(..) | Is::Thermostat |
            Is::Outlet{..} | Is::Shredder{..} | Is::Obstacle(..) | Is::Cabinet(..)
                => (Span::Center, Rise::Center),
            Is::Stair(..) |
            Is::CeilingVent{..} | Is::CeilingDuct{..} | Is::Fan{..} | Is::Candle{..} |
            Is::Grease{..} |
            Is::RubberBands(..) | Is::Clock(..) | Is::Paper(..) | Is::Battery(..) |
            Is::Guitar |
            Is::Teakettle{..} | Is::Fishbowl{..} | Is::Toaster{..} | Is::Ball{..} |
            Is::Books | Is::Basket | Is::Macintosh | Is::Wall(..) 
                => (Span::Center, Rise::Bottom),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum BadObjectError {
    FaultyDimensions(u16, u16, u16, u16),
    OutOfRoom(Bounds),
    UnknownKind(u16),
    NullObject,
}

impl Display for BadObjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FaultyDimensions(l, t, r, b) => write!(f, "object bounding rectangle ({l}, {t}, {r}, {b}) is invalid"),
            Self::OutOfRoom(b) => write!(f, "object bounding rectangle {b:?} extends outside of room"),
            Self::UnknownKind(kind_id) => write!(f, "object declarator \"{kind_id}\" does not indicate a recognized object kind"),
            Self::NullObject => write!(f, "empty object description")
        }
    }
}

impl std::error::Error for BadObjectError {}

impl TryFrom<binary::Object> for Object {
    type Error = BadObjectError;
    fn try_from(value: binary::Object) -> Result<Self, Self::Error> {
        if value.object_is == [0; 2] { return Err(BadObjectError::NullObject) }
        let Ok(bounds): Result<Bounds, _> = value.bounds.try_into() else { 
            let [top, left, bottom, right] = value.bounds.map(u16::from_be_bytes);
            return Err(BadObjectError::FaultyDimensions(left, top, right, bottom))
        };
        if bounds.right() > room::SCREEN_WIDTH || bounds.bottom() > room::SCREEN_HEIGHT { return Err(BadObjectError::OutOfRoom(bounds)) };
        let (amount, extra, ready) = (u16::from_be_bytes(value.amount), u16::from_be_bytes(value.extra), u16::from_be_bytes(value.is_on) != 0);
        use object::Kind;
        let kind = match u16::from_be_bytes(value.object_is) {
             0 => return Err(BadObjectError::NullObject),
             1 => Kind::Table{width: bounds.width(), height: bounds.height()}, 
             
             2 => Kind::Shelf{width: bounds.width(), height: bounds.height()},
             3 => Kind::Books, 
             4 => Kind::Cabinet(bounds.size()),
             5 => Kind::Exit{to: Some(amount.into())},
             6 => Kind::Obstacle(bounds.size()),

             8 => Kind::FloorVent{height:bounds.top() - amount},
             9 => Kind::CeilingVent{height: amount - bounds.bottom()},
            10 => Kind::CeilingDuct{height: amount - bounds.bottom(), destination: Some(extra.into()), ready},
            11 => Kind::Candle{height: bounds.top() - amount},
            12 => Kind::Fan{faces: Side::Left, range: bounds.left() - amount, ready},
            13 => Kind::Fan{faces: Side::Right, range: amount - bounds.right(), ready},

            16 => Kind::Clock(amount),
            17 => Kind::Paper(amount),
            18 => Kind::Grease{range: amount - bounds.right(), ready},
            19 => Kind::Bonus(amount, bounds.size()),
            20 => Kind::Battery(amount),
            21 => Kind::RubberBands(amount),

            24 => Kind::Switch(None),
            25 => Kind::Outlet{delay: amount, ready},
            26 => Kind::Thermostat,
            27 => Kind::Shredder{ready},
            28 => Kind::Switch(Some(amount.into())),
            29 => Kind::Guitar,

            32 => Kind::Drip{range: amount - bounds.top()},
            33 => Kind::Toaster{range: bounds.top() - amount, delay: extra},
            34 => Kind::Ball{range: bounds.bottom() - amount},
            35 => Kind::Fishbowl{range: bounds.y() - amount, delay: extra},
            36 => Kind::Teakettle{delay: amount},
            37 => Kind::Window(bounds.size(), ready),

            40 => Kind::Painting, 
            41 => Kind::Mirror(bounds.size()),
            42 => Kind::Basket,
            43 => Kind::Macintosh,
            44 => Kind::Stair(Vertical::Up, amount.into()),
            45 => Kind::Stair(Vertical::Down, amount.into()),

            bad => return Err( BadObjectError::UnknownKind(bad) )
        };
        Ok(Object{kind, position: bounds * kind.display_anchor()})
        
    }

}


#[derive(Debug)]
pub enum InvalidRoomError {
    Fail,
}

impl<T> From<InvalidRoomError> for Result<T, InvalidRoomError> {
    fn from(value: InvalidRoomError) -> Self { Err(value) }
}

impl From<std::array::TryFromSliceError> for InvalidRoomError {
    fn from(_value: std::array::TryFromSliceError) -> Self {
        Self::Fail
    }
}

impl From<std::convert::Infallible> for InvalidRoomError {
    fn from(_: std::convert::Infallible) -> Self {
        Self::Fail
    }
}

struct EnemyCode(u16);

impl From<EnemyCode> for Option<room::Enemy> {
    fn from(value: EnemyCode) -> Self {
        type Use = room::Enemy;
        Some(match value.0 {
            0 => Use::Dart,
            1 => Use::Copter,
            2 => Use::Balloon,
            _ => return None,
        })
    }
}

impl<T: TryInto<binary::Room>> TryFrom<(room::Id, T)> for Room where InvalidRoomError: From<<T as TryInto<binary::Room>>::Error> {
    type Error = InvalidRoomError;
    fn try_from((id, value): (room::Id, T)) -> Result<Self, Self::Error> {
        use room::On;
        let value = value.try_into()?;
        let header = &value.header;
        let _n_objects@0..=16 = u16::from_be_bytes(header.object_count) else {
            return InvalidRoomError::Fail.into()
        };
        Ok(Self {
            name: string_from_pascal(&header.name),
            back_pict_id: u16::from_be_bytes(header.back_pict_id),
            tile_order: header.tile_order.map(|[_, n]| n),
            left_open: NonZero::new(header.left_right_open[0]).and_then(|_| id.prev()),
            right_open: (header.left_right_open[1] != 0).then_some(()).and_then(|_| id.next()),
            animate: NonZero::new(u16::from_be_bytes(header.animate_number))
                .zip(EnemyCode(u16::from_be_bytes(header.animate_kind)).into())
                .map(|(n, kind)| (kind, n, u32::from_be_bytes(header.animate_delay))),
            environs: On {air: header.condition_code[1] != 1, lights: header.condition_code[1] != 2},
            objects: value.objects.into_iter().filter_map(|o| o.try_into().ok()).collect(),
        })
    }
}

impl<'a> TryFrom<(room::Id, &'a [u8])> for Room {
    type Error = InvalidRoomError;
    fn try_from((id, value): (room::Id, &[u8])) -> Result<Self, Self::Error> {
        Self::try_from((id, *(<&binary::Room>::try_from(value)?)))
    }
}

impl TryFrom<binary::House> for House {
    type Error = <Room as TryFrom<(room::Id, binary::Room)>>::Error;
    fn try_from(value: binary::House) -> Result<Self, Self::Error> {
        let header = &value.header;
        let n_rooms@0..=40 = u16::from_be_bytes(header.n_rooms) as usize else { return InvalidRoomError::Fail.into() };
        Ok(Self {
            version: u16::from_be_bytes(header.version),
            time_stamp: SystemTime::UNIX_EPOCH + Duration::from_secs(u32::from_be_bytes(header.time_stamp) as u64),
            hi_scores: header.hi_scores.iter()
                .zip(header.hi_level)
                .zip(header.hi_names.iter().zip(header.hi_rooms.iter()))
                    .map(|((score, level), (name, room))|
                        Success{
                            score: u32::from_be_bytes(*score),
                            level: u16::from_be_bytes(level),
                            name: string_from_pascal(name),
                            room: string_from_pascal(room),
                        }
                    )
            .collect(),
            pict_file: string_from_pascal(&header.pict_name),
            next_file: string_from_pascal(&header.next_file),
            first_file: string_from_pascal(&header.first_file),
            rooms: value.rooms[..n_rooms]
                .iter().enumerate()
                .map(|(i, r)| Room::try_from((room::Id::from(i), *r))).try_collect()?
        })
    }
}
 

#[cfg(test)]
mod test {
    use super::*;

    const DATA_A: &[u8] = include_bytes!("resources/The House");
    const DATA_B: &[u8] = include_bytes!("resources/The House 2");
    
    fn objects() -> impl Iterator<Item = &'static Block<binary::Object>> {
        [DATA_A, DATA_B].map(|data| data[size_of::<binary::HouseHeader>()..]
            .as_chunks::<{size_of::<binary::Room>()}>().0
            .into_iter()
            .map(|r| 
                r[size_of::<binary::RoomHeader>()..]
                .as_chunks::<{size_of::<binary::Object>()}>()
                .0
            )
            .flatten()
        )
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn rooms() -> impl Iterator<Item = &'static Block<binary::Room>> {
        [DATA_A, DATA_B].map(|data| 
            data[size_of::<binary::HouseHeader>()..].as_chunks::<{size_of::<binary::Room>()}>().0
        )
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .into_iter()       
    }

    #[test]
    fn verify_sizes() {
        assert_eq!(size_of::<binary::Object>(), 16);
        assert_eq!(size_of::<binary::RoomHeader>(), 58);
        assert!(std::mem::align_of::<binary::Room>() <= 2);
        assert_eq!(size_of::<binary::Room>(), 314);
        assert!(std::mem::align_of::<binary::HouseHeader>() <= 2);
        assert_eq!(size_of::<binary::HouseHeader>(), 1270);
        assert_eq!(size_of::<binary::House>(), 13830);
    }

    #[test]
    fn validate_object_bounds() {
        let mut count = 0usize;
        for (index, data) in objects().enumerate() {
            count = index + 1;
            let data = binary::Object::from_iter(*data);
            let (room, obj) = (index / 16 + 1, index % 16 + 1);
            match Object::try_from(data) {
                Err(err@BadObjectError::FaultyDimensions(..)) | 
                Err(err@BadObjectError::OutOfRoom(..))
                    => panic!("object {obj} in room {room} has faulty boundary rectangle :({err})"),
                Err(err@BadObjectError::UnknownKind(..))
                    => panic!("object {obj} in room {room} has unknown object discriminant :({err})"),
                _ => () 
            }
        }
        println!("{count} object bounds verified.");
    }

    #[test]
    fn validate_object_view() {
        let object_bytes = &DATA_A[size_of::<binary::HouseHeader>()..][size_of::<binary::RoomHeader>()..].as_chunks::<{size_of::<binary::Object>()}>().0[0];
        let object: &binary::Object = object_bytes.as_ref();
        assert_eq!(&object.object_is[0], &object_bytes[0]);
    }

    #[test]
    fn validate_room_binary() {
        let test = binary::Room::from_iter((&DATA_A[size_of::<binary::HouseHeader>()..][..size_of::<binary::Room>()]).into_iter().copied());
        let target = &test as *const _ as *const Block<binary::Room>;
        assert!((&unsafe{*target}) == (&DATA_A[size_of::<binary::HouseHeader>()..][..size_of::<binary::Room>()]));
    }

    #[test]
    fn validate_room_binaries() {
        for room_data in rooms() {
            let test = binary::Room::from_iter(room_data.into_iter().copied());
            let target = &test as *const _ as *const Block<binary::Room>;
            assert!(&unsafe{*target} == room_data);
        }
    }

    #[test]
    fn validate_room_passthrough() {
        let room = DATA_A[size_of::<binary::HouseHeader>()..][..size_of::<binary::Room>()].as_chunks().0[0];
        let test = binary::Room::from(room);
        let target = &test as *const _ as *const Block<binary::Room>;
        assert!((&unsafe{*target}) == &room);
    }

    #[test]
    fn validate_house_binary() {
        let test = binary::House::from_iter(DATA_A.into_iter().copied());
        let target = &test as *const _ as *const Block<binary::House>;
        assert!((&unsafe{*target}) == DATA_A);   
    }

    #[test]
    fn validate_house_binaries() {
        for house_data in [DATA_A, DATA_B] {
            let test = binary::House::from_iter(house_data.into_iter().copied());
            let target = &test as *const _ as *const Block<binary::House>;
            assert!((&unsafe{*target}) == house_data);   
        }
    }

    #[test]
    fn validate_house_passthrough() {
        let house = DATA_A.as_chunks().0[0];
        let test = binary::House::from(house);
        let target = &test as *const _ as *const Block<binary::House>;
        assert!((&unsafe{*target}) == &house);
    }
}