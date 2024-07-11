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
    loop {
	    if let result@41.. = unsafe { RAND.borrow_mut().read::<u16>() } { break result }
    }
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
		let size = match self.kind {
            Active::Dart => const{ Size::new(64, 22).unwrap() },
            Active::Copter => const{ Size::new(32, 32).unwrap() },
			Active::Balloon => const{ Size::new(32, 32).unwrap() },
			Active::Flame => const{ Size::new(11, 12).unwrap() },
            Active::Shock => const{ Size::new(32, 25).unwrap() },
            Active::Spill 
                => return Size::new(self.period.start.max(0) as u16, 2)
                .map(|size| (size / (Span::Left, Rise::Bottom) << self.position.as_unsigned())),
			_ => return None
		};
        Some((size / (Span::Center, Rise::Center) << self.position).as_unsigned())
	} 
	fn advance(&mut self, parent_ready: bool) {
        if let Active::Spill = self.kind {
            if self.is_on {eprintln!("Grease spill out to {}", self.period.start);}
        }
        match self.kind {
            Active::Spill if !self.is_on => return,
            Active::Shock if !parent_ready => return,
            _ if self.period.next().is_some() => return,
            Active::Shock if self.is_on => {self.period.start = 0; self.is_on = false; }
            Active::Shock => {self.period.start = self.period.end - 30; self.is_on = true; }
            Active::Dart | Active::Balloon | Active::Copter => {
                self.position += match self.kind {
                    Active::Dart => (-8, 1),
                    Active::Balloon => (0, -3),
                    Active::Copter => (-4, 2),
                    _ => unreachable!(),
                };
                if let None = self.bounds() & room::BOUNDS { self.reset(); }
            }
            #[cfg(debug_assertions)]
            _ => return
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
        Some(match self.kind {
            object::Kind::Candle {..} => Obstacle{
                kind: Active::Flame,
                period: 0..0,
                position: self.position.as_signed() - (3, 27),
                is_on: true,
                control: this.into(),
            },
            object::Kind::Outlet { delay, .. } => Obstacle { 
                kind: Active::Shock, 
                position: self.position.as_signed(), 
                period: 0..(delay as i32), 
                is_on: false,
                control: this.into() 
            },
            object::Kind::Grease { range, .. } => Obstacle{
                kind: Active::Spill,
                position: self.active_area(true).unwrap().as_signed() * (Span::Right, Rise::Bottom) - (0, 1),
                period: -3..(range as i32 + 1),
                is_on: false,
                control: this.into(),
            },
            _ => return None
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
	Escaping(Option<room::Id>),
    Sliding(u16),
    FadingIn(Range<u8>),
    FadingOut(Range<u8>),
    Turning(Side, Range<u8>),
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
            Self::Sliding(..) => None,
            Self::FadingIn(phase)   |
            Self::FadingOut(phase)  |
            Self::Turning(_, phase)  
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

trait Incident {
    fn resolve(&self, player: Bounds, motion: &mut Displacement) -> Option<Event>;
    fn bounds(&self) -> Option<Bounds>;
}

impl From<&State> for u8 {
    fn from(value: &State) -> Self {
        match value {
        	State::Escaping(None) => 0,
        	State::Escaping(_) => 1,
            State::Sliding(..) => 15u8, 
            State::Ascending(..) | State::Descending(..) 
                => 16u8,
            State::FadingIn(phase) => 32u8 + phase.start,
            State::FadingOut(phase) => 48u8 + phase.start,
            //    State::Shredding(_) => 64u8,
            State::Burning(_) => 80u8,
            State::Turning(_, phase) => 96u8 + phase.start,
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
    links: BTreeMap<object::Id, u8>,
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
        let links = Vec::from_iter(
            self.objects.iter().enumerate()
                .filter_map(|(index, o)| {
                    let parent = object::Id::from(index);
                    Some( (parent, id(), o.effect(parent)?) )
                }
            )
        );
        Play {
            room: self,
            score: 0,
            items: BTreeSet::from_iter(self.collider_ids()),
            facing,
            player: (x, y).into(),
            motion: Displacement::default(),
            on: self.environs,
            now: from.action(),
            links: BTreeMap::from_iter(links.iter().map(|&(parent, child, _)| (parent, child))),
            hazards: HashMap::from_iter(
                links.into_iter().map(|(_, id, o)| (id, o))
                .chain(
                    self.animate.map(|(kind, count, delay)| 
                        from_fn(move || kind.new(delay)).take(count.get() as usize)
                    ).into_iter().flatten().map(|h| (id(), h))
                )
            ),
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
            Kind::Fan { faces, .. } => {*h = faces * 7; (faces != state.facing).then_some(Event::Control(State::Turning(faces, 0..11))) }
            Kind::Grease {..} => Some(Event::Action(Update::Start(Environment::Grease, Some(id)), Some(id))),
            Kind::Table{..} | Kind::Shelf{..} | Kind::Books | Kind::Cabinet{..} | Kind::Obstacle{..} | Kind::Basket | 
            Kind::Macintosh | Kind::Drip{..} | Kind::Toaster {..} | Kind::Ball{..} | Kind::Fishbowl {..} 
                => {eprintln!("{:?}", self.kind); Some(Event::Control(DIE))},
            Kind::Clock(value) | Kind::Bonus(value, ..) => Some(Event::Action(Update::Score(value, id), Some(id))),
            Kind::Battery(value) => Some(Event::Action(Update::Energy(value as u8), Some(id))),
            Kind::Paper(_lives) => Some(Event::Action(Update::Life, Some(id))),
            Kind::RubberBands(_bands) => Some(Event::Action(Update::Bands(_bands), Some(id))),
            Kind::FloorVent { .. } | Kind::Candle { .. } => {if state.on.air {*v = -6}; None},
            Kind::Guitar => Some(Event::Action(Update::Start(Environment::Guitar, Some(id)), None)),
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

impl Incident for (object::Id, &object::Object, &Play<'_>) {
    fn resolve(&self, player: Bounds, motion: &mut Displacement) -> Option<Event> {
        self.1.action(player, motion, self.0, self.2)
    }
    fn bounds(&self) -> Option<Bounds> {
        self.1.active_area(self.2.is_ready(self.0))
    }
}

impl Incident for Obstacle {
    fn resolve(&self, player: Bounds, motion: &mut Displacement) -> Option<Event> {
        Some(Event::Control(match self.kind {
            Active::Spill => {
                *motion.y_mut() -= motion.y().saturating_add_unsigned(player.bottom()) - self.position.y();
                State::Sliding(self.position.y().as_unsigned())
            }
            Active::Flame | Active::Shock => IGNITE, 
            _ => DIE
        }))
    }
    fn bounds(&self) -> Option<Bounds> {
        if !self.is_on { return None }
        self.bounds()
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
        let mut signal = self.now.as_ref().map(|s| match s {
            State::FadingIn(..) => vec![Update::Fade(true)],
            State::FadingOut(..) => vec![Update::Fade(false)],
            State::Burning(..) => vec![Update::Burn],
            State::Turning(..) => vec![Update::Turn(self.facing)],
            _ => vec![],
        });
        let control = if let Some(state) = self.now.as_mut() {
            if let Some(motion) = state.next() {
            	let (motion, relative) = motion;
            	let motion = if relative { motion * self.facing } else { motion };
                Some((motion, match state {State::Turning(..) |  State::Burning(..) => true, _ => false}))
            } else {
				let result = state.outcome(self.score);
				self.now = None;
                if let Some(outcome) = result {
                    return outcome
                }
                None
            }
        } else { None };

        if let Some(state) = &self.now {eprintln!("{state:?}");}

        if let (Some((motion, _)), Some(State::Ascending(..))) = (control, &self.now) {
            eprintln!("Asc: {motion:?}");
        }

        let (mut motion, collision) = if let Some(o) = control {
            o
        } else {
            let mut motion = Displacement::new(0, 3);
            for action in actions {
                match action {
                    Input::Go(direction) => *motion.x_mut() += *direction * MAX_THRUST,
                    Input::Flip => {signal.get_or_insert_with(|| vec![Update::Turn(-self.facing)]);},
                    _ => ()
                };
            }
            (motion, true)
        };
        let events = if collision {
            let walls = &BOUNDS[self.room.walls()];
            if let Ok(touch) = Bounds::try_from(PLAYER_SIZE / (Span::Center, Rise::Center) << self.player) {
                let active = BTreeMap::from_iter(self.hazards.iter().map(|(&id, o)| (id, o.control.is_none_or(|parent| self.is_ready(parent)))));
                for (id, hazard) in self.hazards.iter_mut() { 
                    hazard.advance(active[id]); 
                }
                let objects = 
                self.active_entries().map(|(id, o)| (Some(id), o)).chain(walls.iter().map(|w| (None, w)))
                    .map(|(i, o)| (i.unwrap_or(u16::MAX.into()), o, &*self)).collect::<Vec<_>>();
                let actions = objects.iter().map(|o| o as &dyn Incident)
                    .chain(self.hazards.values().map(|h| h as &dyn Incident))
                    .filter_map(|i| (i.bounds() & touch).and_then(|_| i.resolve(touch, &mut motion)))
                    .collect::<Vec<_>>();
                
                let (events, outcomes): (Vec<_>, Vec<_>) = actions.into_iter().map(|e| {
                    match e {
                        Event::Action(a, remove) => {
                            if let Some(ref used) = remove { self.items.remove(used); }
                            match (a, remove) {
                                (Update::Start(Environment::Grease, _), Some(bottle)) => { self.hazards.get_mut(&self.links[&bottle]).map(|o| o.is_on = true); }
                                (Update::Lights, _) => self.on.lights = true,
                                _ => ()
                            };
                            (Some(a), None)
                        },
                        Event::Control(c) => {
                            match c {
                                State::Turning(face, ..) => self.facing = face,
                                _ => ()
                            }
                            (None, Some(c))
                        },
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
        o.get() as usize >= self.room.len() || self.items.contains(&o)
    }
/*
    fn award(&mut self, value: u16) {
        self.score += value as u32;
    }
    */ 

    pub fn dark(&self) -> bool { !self.on.lights }
    pub fn cold(&self) -> bool { !self.on.air }

    pub fn visible_entries(&self) -> impl Iterator<Item = (object::Id, &Object)> {
        self.room.objects.iter().enumerate().filter_map(|(id, o)|{
            let id = id.into();
            match o.kind {
                Kind::Clock(_) |
                Kind::Paper(_) |
                Kind::Battery(_) |
                Kind::RubberBands(_) 
                    => self.items.contains(&id),
                Kind::Grease{..} |
                Kind::Drip{..} |
                Kind::Ball{..} |
                Kind::Fishbowl{..} => true,
                _ => false
            }.then_some((id, o))
        })
    }

     pub fn active_items(&self) -> impl Iterator<Item = &Object> {
        self.items.iter()
            .map(|&index| &self.room[index] )
    }

    pub fn active_entries(&self) -> impl Iterator<Item = (object::Id, &Object)> {
        self.room.objects.iter().enumerate()
            .filter_map(|(id, o)| {let id = id.into(); self.items.contains(&id).then_some((id, o))})
    } 

    pub fn active_hazards(&self) -> impl Iterator<Item = (u16, Active, (i16, i16), bool)> + '_ {
        self.hazards.iter().map(|(&id, Obstacle{kind, position, is_on, period, ..})| {(
            if let Active::Spill = kind {0u16.saturating_add_signed(period.start as i16)} else {id as u16}, 
            *kind, 
            <(i16, i16)>::from(*position), 
            *is_on
        )})
    }

    pub fn player(&self) -> ((i16, i16), Option<Side>, bool) {
        (self.player.into(), match self.now{Some(State::Turning(..)) => None, _ => Some(self.facing)}, self.facing * self.motion.x() < 0)
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

    pub fn debug_zones<'this>(&'this self) -> impl Iterator<Item=Bounds> + 'this {
        self.room.objects.iter().filter_map(|o| o.active_area(true))
    }
}
