use super::*;
use super::room::RoomId;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct ObjectId(pub NonZero<u16>);

impl std::ops::Deref for ObjectId {
    type Target = NonZero<u16>;
    fn deref(&self) -> &Self::Target {
        let ObjectId(ref value) = self;
        value
    }
}

impl From<u16> for ObjectId {
	fn from(value: u16) -> Self { unsafe { Self( NonZero::new_unchecked( value.saturating_sub(1) + 1 ) ) } }
}

impl From<usize> for ObjectId {
	fn from(value: usize) -> Self { unsafe { Self( NonZero::new_unchecked((value + 1) as u16) ) } }
}

impl From<ObjectId> for usize {
	fn from(value: ObjectId) -> Self { value.0.get() as usize - 1 }
}

impl From<ObjectId> for Option<u16> {
    fn from(value: ObjectId) -> Self { Some(value.0.get()) }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectKind {
    Table,
    Shelf,
    Books,
    Cabinet,
    Exit{to: RoomId},
    Obstacle,

    FloorVent{height: u16},
    CeilingVent{height: u16},
    CeilingDuct{height: u16, destination: Option<RoomId>},
    Candle{height: u16},
    Fan{faces: Side, range: u16},

    Clock(u16),
    Paper(u16),
    Grease{range: u16},
    Bonus(u16),
    Battery(u16),
    RubberBands(u16),

    Switch(Option<ObjectId>),
    Outlet{delay: u16},
    Thermostat,
    Shredder,
    Guitar,

    Drip{range: u16},
    Toaster{range: u16, delay: u16},
    Ball{range: u16},
    Fishbowl{range: u16, delay: u16},
    Teakettle{range: u16},
    Window,

    Painting,
    Mirror,
    Basket,
    Macintosh,
    Stair(Vertical, RoomId),

    Wall,
}

#[disclose]
#[derive(Debug, Clone, Copy)]
pub struct Object {
    object_is: ObjectKind,
    bounds: Rect,
    is_on: bool,
}

impl Object {
    pub fn collidable(&self) -> bool {
        match self.object_is { ObjectKind::Painting => false, _ => true }
    }

    pub fn active_area(&self) -> Rect {
        type Kind = ObjectKind;
        match self.object_is {
            Kind::FloorVent { height } | Kind::Candle {height} => Rect{top_: height, bottom_: room::VERT_FLOOR, left_: self.bounds.x() - 8, right_: self.bounds.x() + 8},
            Kind::CeilingVent { height } => Rect{top_: self.bounds.bottom(), bottom_: height, left_: self.bounds.x() - 8, right_: self.bounds.x() + 8},
            Kind::CeilingDuct { height, .. } => if self.is_on {
            	let middle = self.bounds.x(); Rect{left_: middle - 8, right_: middle + 8, top_: room::VERT_CEILING, bottom_: height}
            } else {
            	Rect{bottom_: self.bounds.top_ + 8, ..self.bounds }
            },
            _ => self.bounds
        }
    }

    pub fn dynamic(&self) -> bool {
        match self.object_is {
            ObjectKind::Clock(_) |
            ObjectKind::Paper(_) |
            ObjectKind::Grease{..} |
            ObjectKind::Battery(_) |
            ObjectKind::RubberBands(_) |
            ObjectKind::Drip{..} |
            ObjectKind::Ball{..} |
            ObjectKind::Fishbowl{..} => true,
            _ => false
        }
    }
}
