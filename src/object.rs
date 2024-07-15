pub use motion::Motion;

use super::{*, cart::{Rise, Span}};
use std::num::NonZero;

#[path = "motion.rs"]
mod motion;

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

#[derive(Debug, Clone)]
pub enum Kind {
    Table{width: NonZero<u16>},
    Shelf{width: NonZero<u16>},
    Books,
    Cabinet(Size),
    Exit{to: Option<room::Id>},
    Obstacle(Size),

    Dart(Interval),
    Copter(Interval),
    Balloon(Interval),

    FloorVent{height: u16},
    CeilingVent{height: u16},
    CeilingDuct{height: u16, ready: bool, destination: Option<room::Id>},
    Candle{height: u16},
    Flame,
    Fan{faces: Side, range: u16, ready: bool},

    Clock(u16),
    Paper(u16),
    Grease{range: u16, ready: bool},
    Spill{progress: Interval},
    Bonus(u16, Size),
    Battery(u16),
    RubberBands(u8),
    
    Switch(Option<Id>),
    Outlet{delay: u16, ready: bool},
    Shock{progress: Interval},
    Thermostat,
    Shredder{ready: bool},
    Guitar,
    
    Drip{range: u16},
    Drop(Motion),
    Toaster{range: u16, delay: u16},
    Toast(Motion),
    Bounce{range: u16},
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
            Is::Spill{..} 
                => (Span::Left, Rise::Top),
            Is::Steam{..} 
                => (Span::Right, Rise::Bottom),
            Is::Table{..} | Is::Shelf {..} |
            Is::CeilingVent{..} | Is::CeilingDuct{ready: false, ..} | 
            Is::Drip{..} | Is::Drop(..) |
            Is::Stair(Vertical::Up, ..)
                => (Span::Center, Rise::Top),
            Is::Exit{..} |
            Is::Painting{..} | Is::Mirror(..) | Is::Window(..) |
            Is::Bonus(..) |
            Is::Switch(..) | Is::Thermostat |
            Is::Outlet{..} | Is::Shredder{..} | Is::Obstacle(..) | Is::Cabinet(..) |
            Is::Dart(..) | Is::Copter(..) | Is::Balloon(..) | Is::Flame | Is::Shock{..} | 
            Is::Toast(..) | Is::Fish(..) | Is::Ball(..)
                => (Span::Center, Rise::Center),
            Is::Fan{faces, ..} 
                => (Span::from(-faces), Rise::Center),
            Is::Stair(Vertical::Down, ..) |
            Is::FloorVent{..} | Is::Candle{..} |
            Is::Grease{..} |
            Is::RubberBands(..) | Is::Clock(..) | Is::Paper(..) | Is::Battery(..) |
            Is::Guitar |
            Is::Teakettle{..} | Is::Fishbowl{..} | Is::Toaster{..} | Is::Bounce{..} |
            Is::Books | Is::Basket | Is::Macintosh | 
            Is::CeilingDuct {ready: true, ..}
                => (Span::Center, Rise::Bottom),
            Is::Wall(side) => ((-side).into(), Rise::Bottom)
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
    fn is_ready(&self) -> bool {
        match self.kind {
            Kind::CeilingDuct { ready, .. } |
            Kind::Fan { ready, .. } |
            Kind::Grease { ready, .. } |
            Kind::Outlet { ready, .. } |
            Kind::Shredder { ready } 
                => ready,
            _ => true,
        }
    }

    pub fn is_animated(&self) -> bool {
        match self.kind {
            Kind::Ball(..) | Kind::Balloon(..) | Kind::Copter(..) | Kind::Dart(..) | Kind::Drop(..) |
            Kind::Fish(..) | Kind::Shock{..} | Kind::Spill{..} | Kind::Steam{..} | Kind::Toast(..)
                => true,
            _ => false,
        }
    }

    pub fn collidable(&self) -> bool {
        match self.kind { Kind::Painting | Kind::Outlet { .. } | Kind::Window( .. ) | Kind::Ball{..} => false, _ => self.is_ready() }
    }
 
    pub fn active_area(&self) -> Option<Bounds> {
        let mut position = self.position;
        let mut anchor = self.kind.anchor();
        let size = unsafe{ match self.kind {
            Kind::FloorVent { height } | Kind::Candle {height} => Size::new(16, height.max(1)),
            Kind::CeilingVent { height } => Size::new(16, height),
            Kind::CeilingDuct { height, ready: true, .. } => Size::new(16, height),
            Kind::CeilingDuct{..} => {
                anchor.1 = Rise::Bottom;
                const{ Size::new(48, 13) }
            }
            Kind::Fan{faces, range, ready: true} => {
                match faces {
                    Side::Left => *position.x_mut() -= 17,
                    Side::Right => *position.x_mut() += 17,
                };
                *position.y_mut() -= 44;
                Size::new(range, 20)
            }
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
            Kind::RubberBands(..) => const{ Size::new(32, 23) },
            Kind::Switch(..) => const{ Size::new(18, 26) },
            Kind::Grease {ready: true, ..} => const{ Size::new(32, 29) },
            Kind::Drip {..} => const{ Size::new(16, 13) },
            _ => None
        } }?;
        Some(size / anchor << position)
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
            Kind::Shock{..} |
            Kind::Spill{..} |
            Kind::Toast(..) |
            Kind::Toaster{..} 
                => true,
            _ => false
        }
    }
}
