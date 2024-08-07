pub use motion::Motion;

use super::{*, cart::{Rise, Span}};
use std::num::NonZero;

#[path = "motion.rs"]
mod motion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Id(pub NonZero<usize>);

impl std::ops::Deref for Id {
    type Target = NonZero<usize>;
    fn deref(&self) -> &Self::Target {
        let Id(ref value) = self;
        value
    }
}

impl From<u16> for self::Id {
	fn from(value: u16) -> Self { unsafe { Self( NonZero::new_unchecked( value.saturating_sub(1) as usize + 1 ) ) } }
}

impl From<NonZero<usize>> for self::Id {
    fn from(value: NonZero<usize>) -> Self { Self(value) }
}

impl TryFrom<usize> for self::Id {
    type Error = usize;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(NonZero::new(value).ok_or(value)?))
    }
}

impl From<self::Id> for usize {
	fn from(value: self::Id) -> Self { value.0.get() as usize - 1 }
}

/* impl From<self::Id> for Option<u16> {
    fn from(value: self::Id) -> Self { Some(value.0.get() as u16) }
} */

#[derive(Debug, Clone, Copy)]
pub enum Duct {
    Blow(u16), 
    Travel(Option<room::Id>),
}

#[derive(Debug, Clone)]
pub enum Kind {
    Table{width: NonZero<u16>},
    Shelf{width: NonZero<u16>},
    Books,
    Cabinet(Size),
    Exit{size: Size, to: Option<room::Id>},
    Obstacle(Size),

    Dart(Interval),
    Copter(Interval),
    Balloon(Interval),

    FloorVent{height: u16},
    CeilingVent{height: u16},
    CeilingDuct(Duct),
    Candle{height: u16},
    Flame,
    Fan{faces: Side, range: u16, ready: bool},

    Clock(u16),
    Paper(u16),
    Grease{progress: Interval, ready: bool},
    Bonus(u16, Size),
    Battery(u8),
    RubberBands(u8),
    
    Lights, 
    Switch(Id, Range<i16>),
    Outlet{progress: Interval},
    Thermostat,
    Shredder{ready: bool},
    Guitar,
    
    Drip{range: u16},
    Drop(Motion),
    Toaster{range: u16, delay: u16},
    Toast(Motion, i16),
    Ball(Motion),
    Fishbowl{range: u16, delay: u16},
    Fish(Motion),
    Teakettle{delay: u16},
    Steam{progress: Interval},
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
            Is::Steam{..} 
                => (Span::Right, Rise::Bottom),
            Is::Table{..} | Is::Shelf {..} |
            Is::CeilingVent{..} | Is::CeilingDuct(Duct::Blow(..)) | 
            Is::Drip{..} | Is::Drop(..) |
            Is::Stair(Vertical::Up, ..)
                => (Span::Center, Rise::Top),
            Is::Exit{..} |
            Is::Painting{..} | Is::Mirror(..) | Is::Window(..) |
            Is::Bonus(..) |
            Is::Switch(..) | Is::Lights | Is::Thermostat |
            Is::Outlet{..} | Is::Shredder{..} | Is::Obstacle(..) | Is::Cabinet(..) |
            Is::Dart(..) | Is::Copter(..) | Is::Balloon(..) | Is::Flame | 
            Is::Toast(..) | Is::Fish(..) | Is::Ball(..)
                => (Span::Center, Rise::Center),
            Is::Fan{faces, ..} 
                => (match faces {Side::Left => Span::Right, Side::Right => Span::Left}, Rise::Center),
            Is::Grease{ready: true, ..} => (Span::Right, Rise::Bottom),
            Is::Grease{ready: false, ..} => (Span::Left, Rise::Center),
            Is::Stair(Vertical::Down, ..) |
            Is::FloorVent{..} | Is::Candle{..} |
            Is::RubberBands(..) | Is::Clock(..) | Is::Paper(..) | Is::Battery(..) |
            Is::Guitar |
            Is::Teakettle{..} | Is::Fishbowl{..} | Is::Toaster{..} |
            Is::Books | Is::Basket | Is::Macintosh | 
            Is::CeilingDuct(Duct::Travel(..))
                => (Span::Center, Rise::Bottom),
            Is::Wall(side) => (match side {Side::Left => Span::Right, Side::Right => Span::Left}, Rise::Bottom)
        }
    }
}

#[disclose]
#[derive(Debug, Clone)]
pub struct Object {
    kind: Kind,
    position: Position,
}

impl Object {
    pub fn is_animated(&self) -> bool {
        match self.kind {
            Kind::Ball(..) | Kind::Balloon(..) | Kind::Copter(..) | Kind::Dart(..) | Kind::Drop(..) |
            Kind::Fish(..) | Kind::Outlet{..} | Kind::Grease{..} | Kind::Steam{..} | Kind::Toast(..)
                => true,
            _ => false,
        }
    }
 
    pub fn active_area(&self) -> Option<Bounds> {
        let mut position = self.position;
        let anchor = self.kind.anchor();
        let size = match self.kind {
            Kind::FloorVent { height } | Kind::Candle {height} => Size::new(16, height.max(1)),
            Kind::CeilingVent { height } => Size::new(16, height),
            Kind::CeilingDuct(Duct::Blow(height)) => Size::new(16, height),
            Kind::CeilingDuct{..} => const{ Size::new(48, 13) },
            Kind::Fan{faces, range, ready: true} => {
                *position.x_mut() += faces * 17;
                *position.y_mut() -= 44;
                Size::new(range, 20)
            }
            Kind::Shredder { ready: true } => const{ Size::new(63, 24) },
            Kind::Outlet{progress: Range{start: ..=0, ..}} => const{ Size::new(32, 25) },
            Kind::Stair(v, ..) => { if v == Vertical::Up {*position.y_mut() -= 254}; const{ Size::new(97, 8)}},
            Kind::Wall(..) => const{ Size::new(14, 342) },
            Kind::Table{width} => Some(Size::from((width, const{ NonZero::new(9).unwrap() }))),
            Kind::Shelf{width} => Some(Size::from((width, const{ NonZero::new(5).unwrap() }))),
            Kind::Obstacle(size) |
            Kind::Exit{size, ..} |
            Kind::Bonus(_, size) |
            Kind::Cabinet(size) 
                => Some(size),
            Kind::Books => const{ Size::new(64, 55) },
            Kind::Macintosh => const{ Size::new(45, 58) },
            Kind::Basket => const{ Size::new(63, 71) },
            Kind::Clock(..) => const{ Size::new(32, 29) },
            Kind::Battery(..) => const{ Size::new(16, 26) },
            Kind::Paper(..) => const{ Size::new(48, 21) },
            Kind::RubberBands(..) => const{ Size::new(32, 23) },
            Kind::Lights | Kind::Switch(_, Range{start: 0.., ..}) | Kind::Thermostat => const{ Size::new(18, 27) },
            Kind::Grease {ready: true, ..} => const{ Size::new(32, 29) },
            Kind::Grease {ready: false, progress: Range{start, ..}} if start > 0 => Size::new(start as u16, 2),
            Kind::Dart(..) => const{ Size::new(64, 22) },
            Kind::Ball(..) | Kind::Copter(..) | Kind::Balloon(..) => const{ Size::new(32, 32) },
            Kind::Drop(..) => const{ Size::new(16, 14) },
			Kind::Flame => const{ Size::new(11, 12) },
            Kind::Toast(..) => const{ Size::new(32, 31) },
            Kind::Fish(..) => const{ Size::new(16, 16) },
            Kind::Toaster{..} => { 
                *position.x_mut() += 3;
                *position.y_mut() -= 3;
                const{ Size::new(32, 31) } 
            }
            Kind::Fishbowl{..} => {
                *position.y_mut() += 3;
                const{ Size::new(16, 16) }
            }
            Kind::Steam{progress: Range{start: -10..0, ..}} => const{ Size::new(128, 128) },
            Kind::Guitar => {
                *position.y_mut() -= 56;
                const{ Size::new(2, 90) }
            }
            Kind::Window(..) | Kind::Painting | Kind::Mirror(..) | Kind::Teakettle{..} => None,
            _ => None
        }?;
        Some(size / anchor << *position)
    }

    pub fn is_cosmetic(&self) -> bool {
        match self.kind {
            Kind::Window(..) | Kind::Painting | Kind::Teakettle{..} => true,
            _ => false,
        }
    }

    pub fn is_dynamic(&self) -> bool {
        match self.kind {
            Kind::Clock(_) |
            Kind::Paper(_) |
            Kind::Grease{..} |
            Kind::Battery(_) |
            Kind::RubberBands(_) |
            Kind::Ball{..} |
            Kind::Fish(..) |
            Kind::Fishbowl{..} |
            Kind::Balloon(..) |
            Kind::Copter(..) |
            Kind::Dart(..) |
            Kind::Flame |
            Kind::Drop(..) |
            Kind::Outlet{..} |
            Kind::Toast(..) |
            Kind::Toaster{..} |
            Kind::Lights
                => true,
            _ => false
        }
    }
}
