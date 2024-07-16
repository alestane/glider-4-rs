use crate::{Environment, Position, Reference, Displacement, Size, Bounds, Update, Vertical, cart::{Rise, Span}};

use super::{Input, Outcome, object::{self, Object, Kind, Motion}, room::{self, On, Room}, Side};
use std::{collections::{BTreeMap, BTreeSet}, iter::from_fn, ops::{Index, IndexMut, Range}};

fn random() -> i16 {
	use std::sync::LazyLock;
	use random::Source;
	static mut RAND: LazyLock<std::cell::RefCell<random::Default>> = LazyLock::new(|| std::cell::RefCell::new(random::default(
		match std::time::SystemTime::UNIX_EPOCH.elapsed() {
			Ok(length) => length,
			Err(wrong) => wrong.duration(),
		}.as_secs()
	)));
    unsafe { RAND.borrow_mut().read::<i16>() } 
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

impl object::Kind {
    fn new(&self, delay: i16) -> Option<Object> {
        let (kind, position) = match self {
            Self::Dart(Range{end, ..}) => (Self::Dart(-delay..*end), (544, random() % 150 + 11)),
            Self::Copter(Range{end, ..}) => (Self::Copter(-delay..*end), (random() % 256 + 272, -16)),
            Self::Balloon(Range{end, ..}) => (Self::Balloon(-delay..*end), (random() % 400 + 50, 358)),
            _ => return None,
        };
        Some(Object{kind, position: position.into()})
    }
}
impl Object {
	fn bounds(&self) -> Option<Bounds> {
		let size = match self.kind {
            Kind::Dart(..) => const{ Size::new(64, 22).unwrap() },
            Kind::Copter(..) => const{ Size::new(32, 32).unwrap() },
			Kind::Balloon(..) => const{ Size::new(32, 32).unwrap() },
			Kind::Flame => const{ Size::new(11, 12).unwrap() },
            Kind::Shock{..} => const{ Size::new(32, 25).unwrap() },
            Kind::Drop(ref progress) => return Some(const{ Size::new(16, 14).unwrap() } / (Span::Center, Rise::Top) << (self.position + (0, progress.value() / 32))),
            Kind::Spill{ref progress} 
                => return Size::new(progress.start.max(0) as u16, 2)
                .map(|size| (size / (Span::Left, Rise::Bottom) << self.position)),
			_ => return None
		};
        Some(size / (Span::Center, Rise::Center) << self.position)
	} 
}
 
impl Object {
    fn effect(&self) -> Option<Object> {
        Some(match self.kind {
            Kind::Candle {..} => Object{
                kind: Kind::Flame,
                position: self.position - (3, 27),
            },
            Kind::Outlet { delay, .. } => Object { 
                kind: Kind::Shock { progress: -30..(delay as i16) }, 
                position: self.position, 
            },
            Kind::Grease { range, .. } => Object{
                kind: Kind::Spill { progress: -3..(range as i16 + 1) },
                position: self.active_area()? * (Span::Right, Rise::Bottom) - (0, 1),
            },
            Kind::Drip { range } => Object{
                kind: Kind::Drop(Motion::new(-8, (range as i16) << 5 + 1, 12)),
                position: self.position,
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
    Display(Update), 
    Action(Change),
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

const PLAYER_SIZE: Size = const{ Size::new(28, 10).unwrap() };


pub struct Play<'a> {
    room: &'a Room,
    walls: &'a [Object],
    exits: room::Exits,
    score: u32,
    items: BTreeMap<usize, Object>,
    facing: Side,
    player: Reference,
    motion: Displacement,
    on: On,
    now: Option<State>,
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

    pub fn start(&self, from: Entrance) -> Play {
        eprintln!("{}", self.name);
        for o in &self.objects {
        	eprintln!("{o:?}");
        }
        eprintln!("{:?}", self.environs);
        let mut items = BTreeMap::from_iter(
            self.objects.iter().enumerate().map(|(index, object)| (index, object.clone()))
        );
        let mut spawns = BTreeMap::from_iter(items.iter().filter_map(|(_, host)| 
            host.effect().map(|child| (host as *const _ as usize + 40, child))));
        items.append(&mut spawns);
        items.extend(self.animate.as_ref().map(|(count, kind)| 
            from_fn(move || kind.new(random() % 60 + 30)).take(count.get() as usize)
        ).into_iter().flatten().collect::<Vec<_>>().iter().map(|anim| (&anim as *const _ as usize, anim.clone())));
        let mut this = Play {
            room: self,
            walls: &BOUNDS[self.walls()],
            exits: self.exits,
            score: 0,
            items,
            facing: Side::Left,
            player: (24, 50).into(),
            motion: Displacement::default(),
            on: self.environs,
            now: from.action(),
        };
        this.reset(from);
        this
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
            Kind::Grease {..} => Some(Event::Action(Change::Spill)),
            Kind::Table{..} | Kind::Shelf{..} | Kind::Books | Kind::Cabinet{..} | Kind::Obstacle{..} | Kind::Basket | 
            Kind::Macintosh | Kind::Drip{..} | Kind::Toaster {..} | Kind::Ball{..} | Kind::Fishbowl {..} 
                => {eprintln!("{:?}", self.kind); Some(Event::Control(DIE))},
            Kind::Clock(..) | Kind::Bonus(..) => Some(Event::Action(Change::Collect)),
            Kind::Battery(..) => Some(Event::Action(Change::Collect)),
            Kind::Paper(_lives) => Some(Event::Action(Change::Collect)),
            Kind::RubberBands(_bands) => Some(Event::Action(Change::Collect)),
            Kind::FloorVent { .. } | Kind::Candle { .. } => {if state.on.air {*v = -6}; None},
            Kind::Guitar => Some(Event::Display(Update::Start(Environment::Guitar, Some(id)))),
            Kind::Switch(None) => Some(Event::Action(Change::Light)),
            Kind::Stair(Vertical::Up, to) => Some(Event::Control(State::Ascending(to, state.player.y()))),
            Kind::Stair(Vertical::Down, to) => Some(Event::Control(State::Descending(to, state.player.y()))),
            Kind::Wall{..} => {
                test >>= previous;
                if let Some(bounds) = self.active_area() {
                    if test.left() < bounds.right() && test.right() >= bounds.right() {
                        *h += (bounds.right() - test.left()) as i16;
                    }
                    if test.right() > bounds.left() && test.left() <= bounds.left() {
                        *h -= (test.right() - bounds.left()) as i16;
                    }
                }
                Some(Event::Display(Update::Bump))
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum Change {
    Collect,
    Toggle(object::Id),
    Spill,
    Light,
    Heat,
}

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

        if let (Some((motion, _)), Some(State::Ascending(..))) = (control, &self.now) {
            eprintln!("Asc: {motion:?}");
        }

        let (mut motion, collision) = control.unwrap_or_else(|| {
            let mut motion = Displacement::new(0, 3);
            for action in actions {
                match action {
                    Input::Go(direction) => *motion.x_mut() += *direction * MAX_THRUST,
                    Input::Flip => {signal.get_or_insert_with(|| vec![Update::Turn(-self.facing)]);},
                    _ => ()
                };
            }
            (motion, true)
        });
        let events = if collision {
            if let Ok(touch) = Bounds::try_from(PLAYER_SIZE / (Span::Center, Rise::Center) << self.player) {
                let inactive = self.items.values().filter_map(
                    |host| if let Kind::Outlet{ready: false, ..} | Kind::Grease{ready: false, ..} = host.kind { 
                        Some(host as *const _ as usize + 40)
                    } else { None }
                ).collect::<BTreeSet<_>>();
                for (_, animated) in self.items.iter_mut().filter(|(&index, entity)| !inactive.contains(&index) && entity.is_animated()) {
                    animated.advance();
                }
                let objects = 
                    self.items.iter().map(|(&index, o)| (index, o))
                        .chain(self.walls.iter().map(|wall| (usize::MAX, wall)));
                let incidents = objects
                    .filter_map(|(i, o)| (o.active_area() & touch).and_then(|_| Some((i, o.action(touch, &mut motion, i.into(), self)?))))
                    .collect::<Vec<_>>();

                    let (events, outcomes) = incidents.into_iter().map(|(id, event)| 
                        match event {
                            Event::Display(update) => (Some(update), None),
                            Event::Control(state) => (None, Some(state)),
                            Event::Action(action) => (Some(match action {
                                Change::Heat => {
                                    self.on.air = true;
                                    Update::Air
                                }
                                Change::Light => {
                                    self.on.lights = true;
                                    Update::Lights
                                }
                                Change::Spill => {
                                    match self.get_mut(id) {
                                        Some(Object{kind: Kind::Grease{ready, ..}, ..}) if *ready == true => {*ready = false;}
                                        _ => return (None, None)
                                    }
                                    Update::Start(Environment::Grease, Some(object::Id::from(id as u16)))
                                }
                                Change::Toggle(id) => {
                                    match self.get_child_mut(id) {
                                        Some(Object{kind: Kind::Outlet { ready, .. }, ..}) |
                                        Some(Object{kind: Kind::Shredder { ready }, ..}) |
                                        Some(Object{kind: Kind::CeilingDuct { ready, .. }, ..}) |
                                        Some(Object{kind: Kind::Fan { ready, .. }, ..}) 
                                            => *ready = !*ready,
                                        _ => ()
                                    };
                                    Update::Start(Environment::Switch, None)
                                }
                                Change::Collect => {
                                    if let Some(event) = self.award(id) {
                                        event
                                    } else {
                                        return (None, None)
                                    }
                                }
                            }), None)
                        }
                    )
                    .unzip::<_, _, Vec<_>, Vec<_>>();
                
                let events: Vec<_> = signal.into_iter().flatten().chain(events.into_iter().filter_map(|e| e)).collect();
                self.now = self.now.iter().chain(outcomes.iter().filter_map(|e| e.as_ref())).max().cloned();
                Some(events)
            } else { None }
        } else {
            signal
        }; 
        self.motion = motion;
        self.player += <(i16, i16)>::from(motion);
        if let Some((room::Id(to), out)) = match self.player.x() {..-12 => Some(Side::Left), 489.. => Some(Side::Right), _ => None}.and_then(|s| self.exits[s].zip(Some(s))) {
            return Outcome::Leave{score: self.score, destination: Some((to, Entrance::Flying(-out, self.player.y() as u16)))}
        };
        Outcome::Continue(events)
    }

    pub fn is_ready(&self, o: object::Id) -> bool {
        let index = o.into();
        index >= self.items.len() || self.items.contains_key(&index)
    }

    fn award(&mut self, index: usize) -> Option<Update> {
        let id = object::Id::from(index);
        let ping = match self.get(index)?.kind {
            Kind::Battery(value) => {
                Update::Energy(value, id)
            }
            Kind::Bonus(value, _) |
            Kind::Clock(value) 
                => {
                    Update::Score(value, id)
                }
            Kind::Paper(value) => {
                Update::Life(value, id)
            }
            Kind::RubberBands(count) => {
                Update::Bands(count, id)
            }
            _ => return None
        };
        self.items.remove(&index);
        Some(ping)
    }

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
                    => self.items.contains_key(&id),
                Kind::Grease{..} |
                Kind::Drip{..} |
                Kind::Ball{..} |
                Kind::Fishbowl{..} => true,
                _ => false
            }.then_some((id.into(), o))
        })
    }

     pub fn active_items(&self) -> impl Iterator<Item = &Object> {
        self.items.values()
    }

    pub fn active_entries(&self) -> impl Iterator<Item = (object::Id, &Object)> {
        self.room.objects.iter().enumerate()
            .filter_map(|(index, o)| {let id = index.into(); self.items.get_key_value(&index).map(|(_, o)| (id, o))})
    } 

    pub fn player(&self) -> ((i16, i16), Option<Side>, bool) {
        (self.player.into(), match self.now{Some(State::Turning(..)) => None, _ => Some(self.facing)}, self.facing * self.motion.x() < 0)
    }

    fn entrance(&self, from: Entrance) -> i16 {
        self.room.entrance(from) 
    }

    fn enter_at(&self, from: Entrance) -> ((i16, i16), Side) {
        match from {
        	Entrance::Air => ((self.entrance(from), room::VERT_CEILING as i16 + 10), Side::Right),
            Entrance::Spawn(side) => ((match side { Side::Left => 24, Side::Right => 488}, 50), -side),
            Entrance::Flying(side, height) => ((match side { Side::Left => 24, Side::Right => 488}, height as i16), -side),
            Entrance::Down => ((self.entrance(from) + 88, room::VERT_FLOOR as i16 - 10), Side::Left),
            Entrance::Up => ((self.entrance(from) + 88, room::VERT_CEILING as i16 + 10), Side::Left)
        }
    }
    pub fn reset(&mut self, at: Entrance) {
        let ((x, y), facing) = self.enter_at(at);
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
        self.room.objects.iter().filter_map(|o| o.active_area())
    }

    fn get(&self, index: usize) -> Option<&Object> {
        self.items.get(&index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Object> {
        self.items.get_mut(&index)
    }

    fn get_child(&self, object::Id(index): object::Id) -> Option<&Object> {
        self.get(self.get(index.get() as usize)? as *const _ as usize + 40)
    }

    fn get_child_mut(&mut self, object::Id(index): object::Id) -> Option<&mut Object> {
        self.get_mut(self.get(index.get() as usize)? as *const _ as usize + 40)
    }
}