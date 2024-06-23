use super::*;
use std::num::NonZero;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Id(pub NonZero<u16>);

impl std::ops::Deref for Id {
    type Target = NonZero<u16>;
    fn deref(&self) -> &Self::Target {
        let Id(ref value) = self;
        value
    }
}

impl From<u16> for self::Id {
	fn from(value: u16) -> Self { unsafe { Self( NonZero::new_unchecked( value.saturating_sub(1) + 1 ) ) } }
}

impl From<usize> for self::Id {
	fn from(value: usize) -> Self { unsafe { Self( NonZero::new_unchecked((value + 1) as u16) ) } }
}

impl From<self::Id> for usize {
	fn from(value: self::Id) -> Self { value.0.get() as usize - 1 }
}

impl From<self::Id> for Option<u16> {
    fn from(value: self::Id) -> Self { Some(value.0.get()) }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Table{width: NonZero<u16>, height: NonZero<u16>},
    Shelf{width: NonZero<u16>, height: NonZero<u16>},
    Books,
    Cabinet(Size),
    Exit{to: Option<room::Id>},
    Obstacle(Size),

    FloorVent{height: u16},
    CeilingVent{height: u16},
    CeilingDuct{height: u16, ready: bool, destination: Option<room::Id>},
    Candle{height: u16},
    Fan{faces: Side, range: u16, ready: bool},

    Clock(u16),
    Paper(u16),
    Grease{range: u16, ready: bool},
    Bonus(u16, Size),
    Battery(u16),
    RubberBands(u16),

    Switch(Option<Id>),
    Outlet{delay: u16, ready: bool},
    Thermostat,
    Shredder{ready: bool},
    Guitar,

    Drip{range: u16},
    Toaster{range: u16, delay: u16},
    Ball{range: u16},
    Fishbowl{range: u16, delay: u16},
    Teakettle{range: u16},
    Window(Size, bool),

    Painting,
    Mirror(Size),
    Basket,
    Macintosh,
    Stair(Vertical, room::Id),

    Wall(Side),
}

#[disclose]
#[derive(Debug, Clone, Copy)]
pub struct Object {
    kind: Kind,
    position: Point<u16>,
}

impl Object {
    pub fn collidable(&self) -> bool {
        match self.kind { Kind::Painting | Kind::Outlet { .. } | Kind::Window( .. ) | Kind::Ball{..} => false, _ => true }
    }
 
    pub fn active_area(&self) -> Rect<u16> {
        let position = self.position;
        match self.kind {
            // Kind::FloorVent { height } | Kind::Candle {height} => Rect::new_forced(position.x() - 8, position.y() - height, position.x() + 8, position.y()),
            // Kind::CeilingVent { height } => Rect{top_: bounds.bottom(), bottom_: height, left_: bounds.x() - 8, right_: bounds.x() + 8},
            // Kind::CeilingDuct { height, .. } => if self.is_on {
            // 	let middle = bounds.x(); Rect{left_: middle - 8, right_: middle + 8, top_: room::VERT_CEILING, bottom_: height}
            // } else {
            // 	Rect{bottom_: bounds.top_ + 8, ..bounds }
            // },
            // Kind::Fan { faces: Side::Right, range } => Rect{left_: bounds.right(), top_: bounds.top() + 10, right_: range, bottom_: bounds.top() + 30},
            // Kind::Fan { faces: Side::Left, range } => Rect{left_: range, top_: bounds.top() + 10, right_: bounds.left(), bottom_: bounds.top() + 30},
            // Kind::Stair(Vertical::Up, ..) => Rect{left_: bounds.left() + 32, top_: bounds.top(), right_: bounds.right() - 32, bottom_: bounds.top() + 8},
            // Kind::Stair(Vertical::Down, ..) => Rect{left_: bounds.left() + 32, top_: bounds.bottom() - 8, right_: bounds.right() - 32, bottom_: bounds.bottom()},
            _ => unsafe { Rect::clamped_on(self.position.into(), NonZero::new_unchecked(1), NonZero::new_unchecked(1)) }
        }
    }

    pub fn dynamic(&self) -> bool {
        match self.kind {
            Kind::Clock(_) |
            Kind::Paper(_) |
            Kind::Grease{..} |
            Kind::Battery(_) |
            Kind::RubberBands(_) |
            Kind::Drip{..} |
            Kind::Ball{..} |
            Kind::Fishbowl{..} => true,
            _ => false
        }
    }
}
