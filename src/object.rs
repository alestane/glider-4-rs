use cart::Transfer;

use super::{*, cart::{Rise, Span}};
use std::num::NonZero;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    Table{width: NonZero<u16>},
    Shelf{width: NonZero<u16>},
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
    Teakettle{delay: u16},
    Window(Size, bool),

    Painting,
    Mirror(Size),
    Basket,
    Macintosh,
    Stair(Vertical, room::Id),

    Wall(Side),
}

impl Kind {
    pub(super) const fn anchor(&self) -> (Span, Rise) {
        type Is = Kind;
        match self {
            Is::Table{..} | Is::Shelf {..} |
            Is::CeilingVent{..} | Is::CeilingDuct{ready: false, ..} | 
            Is::Drip{..} |
            Is::Stair(Vertical::Up, ..)
                => (Span::Center, Rise::Top),
            Is::Exit{..} |
            Is::Painting{..} | Is::Mirror(..) | Is::Window(..) |
            Is::Bonus(..) |
            Is::Switch(..) | Is::Thermostat |
            Is::Outlet{..} | Is::Shredder{..} | Is::Obstacle(..) | Is::Cabinet(..)
                => (Span::Center, Rise::Center),
            Is::Stair(Vertical::Down, ..) |
            Is::Fan{..} | 
            Is::FloorVent{..} | Is::Candle{..} |
            Is::Grease{..} |
            Is::RubberBands(..) | Is::Clock(..) | Is::Paper(..) | Is::Battery(..) |
            Is::Guitar |
            Is::Teakettle{..} | Is::Fishbowl{..} | Is::Toaster{..} | Is::Ball{..} |
            Is::Books | Is::Basket | Is::Macintosh | 
            Is::CeilingDuct {ready: true, ..}
                => (Span::Center, Rise::Bottom),
            Is::Wall(side) => ((-side).into(), Rise::Bottom)
        }
    }
}

#[disclose]
#[derive(Debug, Clone, Copy)]
pub struct Object {
    kind: Kind,
    position: Position,
}

impl Object {
    pub fn collidable(&self) -> bool {
        match self.kind { Kind::Painting | Kind::Outlet { .. } | Kind::Window( .. ) | Kind::Ball{..} => false, _ => true }
    }
 
    pub fn active_area(&self, ready: bool) -> Option<Bounds> {
        let mut position = self.position;
        let anchor = self.kind.anchor();
        let size = unsafe{ match self.kind {
            Kind::FloorVent { height } | Kind::Candle {height} => Size::new(16, height.max(1)),
            Kind::CeilingVent { height } => Size::new(16, height),
            Kind::CeilingDuct { height, .. } => return Some((if ready {
                    Size::new(16, height).unwrap() / (Span::Center, Rise::Bottom)
                } else {
                    (const{ Size::new_unchecked(48, 13) }) / (Span::Center, Rise::Top) 
                } << position).as_unsigned()),
            // Kind::Fan { faces: Side::Right, range } => Rect{left_: bounds.right(), top_: bounds.top() + 10, right_: range, bottom_: bounds.top() + 30},
            // Kind::Fan { faces: Side::Left, range } => Rect{left_: range, top_: bounds.top() + 10, right_: bounds.left(), bottom_: bounds.top() + 30},
            Kind::Stair(v, ..) => { if v == Vertical::Up {*position.y_mut() -= 254}; const{ Size::new(97, 8)}},
            Kind::Wall(..) => const{ Size::new(14, 342) },
            Kind::Obstacle(size) => Some(size),
            Kind::Table{width} => Some(Size::from((width, const{ NonZero::new_unchecked(9) }))),
            Kind::Shelf{width} => Some(Size::from((width, const{ NonZero::new_unchecked(5) }))),
            Kind::Cabinet(size) => Some(size),
            Kind::Books => const{ Size::new(64, 55) },
            Kind::Macintosh => const{ Size::new(45, 58) },
            Kind::Clock(..) => const{ Size::new(32, 29) },
            Kind::Battery(..) => const{ Size::new(16, 26) },
            Kind::Paper(..) => const{ Size::new(48, 21) },
            #[cfg(debug_assertions)]
            _ => None
        } }?;
        Some((size / anchor << position).as_unsigned())
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
