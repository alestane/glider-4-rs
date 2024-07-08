use crate::{Environment, Position, Reference, Displacement, Size, Bounds, Update, Vertical, cart::{Rise, Span, Transfer}};

use super::{Input, Outcome, object::{self, Object, Kind}, room::{self, On, Room, Active}, Side};
use std::{collections::{BTreeMap, BTreeSet, HashMap}, iter::from_fn, num::NonZero, ops::Range};


fn random() -> u16 {
	use std::sync::LazyLock;
	use random::Source;
	static mut RAND: LazyLock<std::cell::RefCell<random::Default>> = LazyLock::new(|| std::cell::RefCell::new(random::default(
		match std::time::SystemTime::UNIX_EPOCH.elapsed() {
			Ok(length) => length,
			Err(wrong) => wrong.duration(),
		}.as_secs()
	)));
	unsafe { RAND.borrow_mut().read::<u16>() }
}

const MAX_THRUST: i16 = 5;

#[derive(Debug, Clone, Copy)]
pub enum Entrance {
    Spawn(Side),
    Flying(Side, u16),
    Up, 
    Down,
	Air,
}

impl Default for Entrance {
    fn default() -> Self { Self::Spawn(Side::Left) }
}

impl Entrance {
    fn action(&self) -> Option<State> {
        Some(match self {
            Self::Spawn(..) => State::FadingIn(0..16),
            _ => return None
        })
    }
}

#[derive(Debug, Clone)]
struct Obstacle {
    kind: Active,
    position: Reference,
    period: Range<i32>,
    is_on: bool,
    control: Option<object::Id>
}
impl Active {
    fn new(&self, delay: u32) -> Option<Obstacle> {
        Some(Obstacle {
            kind: *self,
			position: if let Some(start) = self.start() {start} else {return None},
			period: Self::period(delay),
			is_on: true,
            control: None,
            })
    }
	fn start(&self) -> Option<Reference> {
		Some(match self {
            Self::Dart => (544, (random() % 150) as i16 + 11),
			Self::Balloon => ((random() % 400)  as i16 + 50, 358),
            Self::Copter => ((random() % 256) as i16 + 272, -16),
			_ => return None
        }.into())
    }
	fn period(delay: u32) -> Range<i32> {
		let delay = delay as i32;
		(delay - (random() as i32 % (delay + 60) + 30))..delay
    }
}
impl Obstacle {
	fn bounds(&self) -> Option<Bounds> {
		let (width, height) = unsafe { match self.kind {
            Active::Dart => (NonZero::new_unchecked(64), NonZero::new_unchecked(22)),
            Active::Copter => (NonZero::new_unchecked(32), NonZero::new_unchecked(32)),
			Active::Balloon => (NonZero::new_unchecked(32), NonZero::new_unchecked(32)),
			Active::Flame => (NonZero::new_unchecked(11), NonZero::new_unchecked(12)),
            Active::Shock => (NonZero::new_unchecked(32), NonZero::new_unchecked(25)), 
			_ => (NonZero::new_unchecked(1), NonZero::new_unchecked(1))
		} };
        Some(((Size::from((width, height)) / (Span::Center, Rise::Center)) << self.position).as_unsigned())
	} 
	fn advance(&mut self) {
        if self.period.next().is_none() {
            self.position += match self.kind {
                Active::Dart => (-8, 1),
                Active::Balloon => (0, -3),
                Active::Copter => (-4, 2),
                _ => (0, 0),
            };
            match self.kind {
                Active::Dart | Active::Balloon | Active::Copter => if let None = self.bounds().map(|bounds| bounds & room::BOUNDS) { self.reset(); }
                Active::Shock => if self.is_on { 
                    self.period.start = 0; self.is_on = false; 
                } else { 
                    self.period.start = self.period.end - 30; self.is_on = true; 
                },
                _ => ()
            }
            
        };
	} 
	fn reset(&mut self) {
		let delay = self.period.end;
		match (self.kind, self.kind.start()) {
			(_, Some(start)) => {
				self.period.start = delay - (random() as i32 % (delay + 60) + 30);
				self.position = start;
			},
			(Active::Flame, _) => (),
			_ => return,
		}
	}
}
 
impl Object {
    fn effect(&self, this: object::Id) -> Option<Obstacle> {
        Option::<Active>::from(self.kind).and_then(|kind|
            Some(match kind {
                Active::Flame => Obstacle{
                    kind, 
                    period: 0..0, 
                    position: self.position.as_signed() - (3, 27), 
                    is_on: true, 
                    control: this.into()
                },
                Active::Shock => Obstacle{
                    kind, 
                    period: if let object::Kind::Outlet { delay, .. } = self.kind {0..(delay as i32)} else {return None}, 
                    position: self.position.as_signed(),
                    is_on: false, 
                    control: this.into()
                },
                _ => return None
            })
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
	Escaping(Option<room::Id>),
    FadingIn(Range<u8>),
    FadingOut(Range<u8>),
    Turning(Range<u8>),
//    Shredding(Rect),
    Burning(Range<u16>),
    Ascending(room::Id, i16),
   Descending(room::Id, i16),
}

const DIE: State = State::FadingOut(0..16);
const IGNITE: State = State::Burning(0..150);
impl State {
    fn outcome(&self, score: u32) -> Option<Outcome> {
        Some(match self {
			Self::Escaping(to) => Outcome::Leave { score, destination: to.map(|room::Id(id)| (id, Entrance::Air))},
            Self::FadingOut(_) | Self::Burning(_) /* | Self::Shredding(_) */ => Outcome::Dead,
            Self::Ascending(room::Id(room), _) /*| Self::Descending(RoomId(room), _)*/
                 => Outcome::Leave{score, destination: Some((*room, Entrance::Down))},
            Self::Descending(room::Id(room), _)
                => Outcome::Leave{score, destination: Some((*room, Entrance::Up))},
            _ => return None
        })
    }
}

impl std::iter::Iterator for State {
    type Item = (Displacement, bool);
    fn next(&mut self) -> Option<Self::Item> {
        match self {
        	Self::Escaping(..) => None,
            Self::FadingIn(phase)   |
            Self::FadingOut(phase)  |
            Self::Turning(phase)  
                => phase.next().map(|_| (Displacement::default(), false)),
            Self::Burning(phase) => {if phase.next().is_none() {eprintln!("burn timeout"); *self = DIE}; Some(((1, 3).into(), true)) },
            /* Self::Shredding(bounds) => match bounds.height().get() {
                0..36 => {bounds._bottom += 1; Some((0, (bounds.height().get() % 2) as i16))},
                _ => {bounds._top += 8; bounds._bottom += 8; (bounds._top > 342).then_some((0, 8))}
            }, */
            Self::Ascending(_, v) => {*v -= 6; (*v >= 230).then_some(((-2, -6).into(), false))}
            Self::Descending(_, v) => {*v += 6; (*v <= 130).then_some(((2, 6).into(), false))},
        }
    }
}
#[derive(Debug, Clone)]
enum Event {
    Control(State),
    Action(Update, Option<object::Id>),
}

impl From<&State> for u8 {
    fn from(value: &State) -> Self {
        match value {
        	State::Escaping(None) => 0,
        	State::Escaping(_) => 1,
            State::Ascending(..) | State::Descending(..) 
                => 16u8,
            State::FadingIn(phase) => 32u8 + phase.start,
            State::FadingOut(phase) => 48u8 + phase.start,
            //    State::Shredding(_) => 64u8,
            State::Burning(_) => 80u8,
            State::Turning(phase) => 96u8 + phase.start,
        }
    }
}

impl std::cmp::PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        u8::from(other).cmp(&u8::from(self))
    }
}

fn id() -> u8 {
    static mut NEXT: NonZero<u8> = unsafe { NonZero::new_unchecked(73) };
    let id = unsafe { NEXT.get() };
    unsafe { NEXT = NonZero::new(id.wrapping_add(73)).unwrap_or(NonZero::new_unchecked(73) ) };
    id
}

const PLAYER_SIZE: Size = unsafe{ Size::new_unchecked(28, 10) };

pub struct Play<'a> {
    room: &'a Room,
    score: u32,
    items: BTreeSet<object::Id>,
    facing: Side,
    player: Reference,
    motion: Displacement,
    on: On,
    now: Option<State>,
    ready: BTreeMap<object::Id, bool>,
    hazards: HashMap<u8, Obstacle>,
}
impl Room {
    pub fn collider_ids(&self) -> impl Iterator<Item = object::Id> + '_ {
        self.objects.iter().enumerate().filter_map(|(id, o)| o.collidable().then_some(id.into()))
    }
    
    fn entrance(&self, from: Entrance) -> i16 {
        fn is_active_duct(o: &&Object) -> bool { matches!(o.kind, object::Kind::CeilingDuct { .. }) }
        fn is_down_stair(o: &&Object) -> bool { matches!(o.kind, object::Kind::Stair(Vertical::Down, _)) }
        self.objects.iter()
        .filter(
            match from {
                Entrance::Air => is_active_duct,
                Entrance::Down => is_down_stair,
                _ => return 232
            }
        )
        .map(|o| o.position.x() as i16).last().unwrap_or(232)
    }

    fn enter_at(&self, from: Entrance) -> ((i16, i16), Side) {
        match from {
        	Entrance::Air => ((self.entrance(from), room::VERT_CEILING as i16 + 10), Side::Right),
            Entrance::Spawn(side) => ((match side { Side::Left => 24, Side::Right => 488}, 50), -side),
            Entrance::Flying(side, height) => ((match side { Side::Left => 24, Side::Right => 488}, height as i16), -side),
            Entrance::Down => ((self.entrance(from) + 88, room::VERT_FLOOR as i16 - 10), Side::Left),
            Entrance::Up => ((self.entrance(from) + 88, room::VERT_CEILING as i16 + 10), Side::Left)
//            Entrance::Appearing(target) => {let bounds = self.objects[target as usize].bounds; (bounds.x(), bounds.y(), Side::Right)}
        }
    }

    pub fn start(&self, from: Entrance) -> Play {
        let ((x, y), facing) = self.enter_at(from);
        for o in &self.objects {
        	eprintln!("{o:?}");
        }
        eprintln!("{:?}", self.environs);
        Play {
            room: self,
            score: 0,
            items: BTreeSet::from_iter(self.collider_ids()),
            facing,
            player: (x, y).into(),
            motion: Displacement::default(),
            on: self.environs,
            ready: BTreeMap::new(),
            now: from.action(),
            hazards: HashMap::from_iter(self.objects.iter().enumerate()
            	.filter_map(|(id, o)| o.effect(id.into()))
            	.chain(self.animate.map(|(kind, count, delay)| from_fn(move || kind.new(delay)).take(count.get() as usize)).into_iter().flatten())
            	.map(|h| (id(), h))),
        }
    }
}

impl super::object::Object {
    fn action(&self, mut test: Bounds, motion: &mut Displacement, id: object::Id, state: &Play) -> Option<Event> {
        use object::Kind;
        let previous = *motion;
        let (h, v) = motion.as_mut();
        match self.kind {
            Kind::CeilingDuct { destination, .. } if !state.is_ready(id) => Some(Event::Control(State::Escaping(destination))),
            Kind::CeilingDuct {..} | Kind::CeilingVent {..} => {if state.on.air {*v = 8}; None},
            Kind::Fan { faces, .. } => {*h = faces * 7; (faces != state.facing).then_some(Event::Control(State::Turning(0..11))) }
            Kind::Table{..} | Kind::Shelf{..} | Kind::Books | Kind::Cabinet{..} | Kind::Obstacle{..} | Kind::Basket | 
            Kind::Macintosh | Kind::Drip{..} | Kind::Toaster {..} | Kind::Ball{..} | Kind::Fishbowl {..} 
                => Some(Event::Control(DIE)),
            Kind::Clock(value) | Kind::Bonus(value, ..) => Some(Event::Action(Update::Score(value), Some(id))),
            Kind::Battery(value) => Some(Event::Action(Update::Energy(value as u8), Some(id))),
            Kind::Paper(_lives) => Some(Event::Action(Update::Life, Some(id))),
            Kind::FloorVent { .. } | Kind::Candle { .. } => {if state.on.air {*v = -6}; None},
            Kind::Guitar => Some(Event::Action(Update::Start(Environment::Guitar), None)),
            Kind::Switch(None) => Some(Event::Action(Update::Lights, None)),
            Kind::Stair(Vertical::Up, to) => Some(Event::Control(State::Ascending(to, state.player.y()))),
            Kind::Stair(Vertical::Down, to) => Some(Event::Control(State::Descending(to, state.player.y()))),
            Kind::Wall{..} => {
                test >>= previous;
                if let Some(bounds) = self.active_area(true) {
                    if test.left() < bounds.right() && test.right() >= bounds.right() {
                        *h += (bounds.right() - test.left()) as i16;
                    }
                    if test.right() > bounds.left() && test.left() <= bounds.left() {
                        *h -= (test.right() - bounds.left()) as i16;
                    }
                }
                Some(Event::Action(Update::Bump, None))
            }
            _ => None
        }
    }
}

const BOUNDS: [Object; 3] = [
    Object{
        kind: object::Kind::Wall(Side::Left),
        position: Position::new(14, 342),
    },
    Object{
        kind: object::Kind::Obstacle(unsafe {Size::new_unchecked(512, 17)}),
        position: Position::new(room::SCREEN_WIDTH / 2, room::VERT_FLOOR + 8),
    },
    Object{
        kind: object::Kind::Wall(Side::Right),
        position: Position::new(498, 342),
    },
];

 impl<'a> Play<'a> {
    pub fn frame(&mut self, actions: &[Input]) -> Outcome {
        let signal = self.now.as_ref().map(|s| match s {
            State::FadingIn(..) => vec![Update::Fade(true)],
            State::FadingOut(..) => vec![Update::Fade(false)],
            State::Burning(..) => vec![Update::Burn],
            _ => vec![],
        });
        let control = if let Some(state) = self.now.as_mut() {
            if let Some(motion) = state.next() {
            	let (motion, relative) = motion;
            	let motion = if relative { motion * self.facing } else { motion };
                Some((motion, match state {/* State::Turning(_) | */ State::Burning(..) => true, _ => false}))
            } else {
				let result = state.outcome(self.score);
				self.now = None;
                if let Some(outcome) = result {
                    return outcome
                }
                None
            }
        } else { None };

        let (mut motion, collision) = if let Some(o) = control {
            o
        } else {
            let mut motion = Displacement::new(0, 3);
            for action in actions {
                match action {
                    Input::Go(direction) => *motion.x_mut() += *direction * MAX_THRUST,
                    _ => ()
                };
            }
            (motion, true)
        };
        let events = if collision {
            let walls = &BOUNDS[self.room.walls()];
            if let Ok(touch) = Bounds::try_from(PLAYER_SIZE / (Span::Center, Rise::Center) << self.player) {
                for hazard in self.hazards.values_mut() { hazard.advance(); }
                let actions: Vec<_> = self.active_items().chain(walls).enumerate().filter_map(|(i, o)|
                    {
                        let i = i.into();
                        (o.active_area(self.is_ready(i)) & touch).and_then(|_| o.action(touch, &mut motion, i, self))
                    }
                ).chain(self.hazards.values().filter_map(|h|
                    h.is_on
                        .then_some(h)
                        .and_then(Obstacle::bounds)
                        .and_then(|bounds| (bounds & touch).map(|_| Event::Control(match h.kind {Active::Flame | Active::Shock => IGNITE, _ => DIE})))
                )).collect();
                let (events, outcomes): (Vec<_>, Vec<_>) = actions.into_iter().map(|e| {
                    match e {
                        Event::Action(a, remove) => {
                            if let Some(ref used) = remove {self.items.remove(used);};
                            match a {
                                Update::Lights => self.on.lights = true,
                                _ => ()
                            };
                            (Some(a), None)
                        },
                        Event::Control(c) => (None, Some(c)),
                    }
                }).unzip();
                let events: Vec<_> = signal.into_iter().flatten().chain(events.into_iter().filter_map(|e| e)).collect();
                self.now = self.now.iter().chain(outcomes.iter().filter_map(|e| e.as_ref())).max().cloned();
                Some(events)
            } else { None }
        } else {
            signal
        }; 
        self.motion = motion;
        self.player += <(i16, i16)>::from(motion);
        if let Some((room::Id(to), out)) = match self.player.x() {..-12 => Some(Side::Left), 489.. => Some(Side::Right), _ => None}.and_then(|s| self.room[s].zip(Some(s))) {
            return Outcome::Leave{score: self.score, destination: Some((to, Entrance::Flying(-out, self.player.y() as u16)))}
        };
        Outcome::Continue(events)
    }

    pub fn is_ready(&self, o: object::Id) -> bool {
        self.ready.get(&o).map(|&ready| ready).unwrap_or_else(|| 
            if o.get() as usize >= self.room.len() { true } else {
                match self.room[o].kind {
                    Kind::CeilingDuct { ready, .. } |
                    Kind::Fan { ready, .. } |
                    Kind::Grease { ready, .. } |
                    Kind::Outlet { ready, .. } |
                    Kind::Shredder { ready } 
                        => ready,
                    _ => true,
                }
            }
        )
    }
/*
    fn award(&mut self, value: u16) {
        self.score += value as u32;
    }
    */ 

    pub fn dark(&self) -> bool { !self.on.lights }
    pub fn cold(&self) -> bool { !self.on.air }

    pub fn active_items(&self) -> impl Iterator<Item = &Object> {
        self.items.iter()
            .map(|&index| &self.room[index] )
    }

    pub fn active_entries(&self) -> impl Iterator<Item = (object::Id, &Object)> {
        self.items.iter()
            .map(|&index| (index, &self.room[index]) )
    }

    pub fn active_hazards(&self) -> impl Iterator<Item = (u8, Active, (i16, i16), bool)> + '_ {
        self.hazards.iter().map(|(&id, Obstacle{kind, position, is_on, ..})| (id, *kind, <(i16, i16)>::from(*position), *is_on))
    }

    pub fn player(&self) -> ((i16, i16), Side, bool) {
        (self.player.into(), self.facing, self.facing * self.motion.x() < 0)
    }

    pub fn reset(&mut self, at: Entrance) {
        let ((x, y), facing) = self.room.enter_at(at);
        if !matches!(at, Entrance::Air) {
			self.facing = facing;
        }
        self.player = Reference::new(x, y);
        self.motion = Displacement::default();
        if let Entrance::Spawn(..) = at {
        	self.now = Some(State::FadingIn(0..16));
        }
    }
}
