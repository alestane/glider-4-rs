use crate::{Environment, Position, Reference, Displacement, Size, Bounds, Update, Vertical, cart::{Rise, Span}, prelude::{Blow, Travel}};

use super::{Input, Outcome, object::{self, Object, Kind, Motion}, room::{self, On, Room}, Side};
use std::{iter::{from_fn, once}, num::NonZero, ops::{Index, IndexMut, Range}};

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
            Self::Down => State::Ascending(room::VERT_FLOOR as i16 - 10),
            Self::Up => State::Descending(room::VERT_CEILING as i16 + 10),
            _ => return None
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
	Escaping(Option<room::Id>, Range<u8>),
    Landed,
    Sliding(i16),
    FadingIn(Range<u8>),
    FadingOut(Range<u8>),
    Turning(Side, Range<u8>),
    Shredding{height: u16, x: i16, top: i16},
    Burning(Range<u16>),
    Stairs(Vertical, room::Id),
    Ascending(i16),
    Descending(i16),
}

const DIE: State = State::FadingOut(0..16);
const IGNITE: State = State::Burning(0..150);
impl State {
    fn outcome(&self, score: u32) -> Option<Outcome> {
        Some(match self {
			Self::Escaping(to, time) if time.start >= time.end => Outcome::Leave { score, destination: to.map(|room::Id(id)| (id, Entrance::Air))},
            Self::Stairs(Vertical::Up, destination) => Outcome::Leave{score, destination: Some((destination.0, Entrance::Down)) },
            Self::Stairs(Vertical::Down, destination) => Outcome::Leave{score, destination: Some((destination.0, Entrance::Up))},
            Self::FadingOut(_) | Self::Burning(_) | Self::Shredding{..} => Outcome::Dead,
            _ => return None
        })
    }
}

impl std::iter::Iterator for State {
    type Item = (Displacement, bool);
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Sliding(..) => None,
            Self::Landed => {
                *self = Self::FadingOut(1..16);
                Some((Displacement::default(), false))
            }
        	Self::Escaping(_, phase) |
            Self::FadingIn(phase)   |
            Self::FadingOut(phase)  |
            Self::Turning(_, phase)  
                => phase.next().map(|_| (Displacement::default(), false)),
            Self::Burning(phase) => {if phase.next().is_none() {eprintln!("burn timeout"); *self = DIE}; Some(((1i16, 3i16).into(), true)) },
            Self::Shredding{height, top, ..} => {
                if *top > 342 {return None}
                Some(((0i16, match height {
                    ..36 => {*height += 1; 0}
                    _ => {*top += 8; 8}
                }).into(), false))
            },
            Self::Stairs(..) => None,
            Self::Ascending(v) => {*v -= 6; (*v >= 230).then_some(((-2i16, -6i16).into(), false))}
            Self::Descending(v) => {*v += 6; (*v <= 130).then_some(((2i16, 6i16).into(), false))},
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
            State::Shredding{..} => 0,
        	State::Escaping(None, _) => 1,
        	State::Escaping(..) | State::Stairs(..) => 2,
            State::Landed => 48,
            State::Sliding(..) => 15u8, 
            State::Ascending(..) | State::Descending(..) 
                => 16u8,
            State::FadingIn(phase) => 32u8 + phase.start,
            State::FadingOut(phase) => 4u8 + phase.start / 2,
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

mod iter {
    use std::{iter::{FilterMap, self}, slice};
    use super::{Object, object};
    pub type Enumerate<'a> = FilterMap<iter::Enumerate<slice::Iter<'a, Option<Object>>>, fn((usize, &Option<Object>)) -> Option<(object::Id, &Object)>>;
    pub type Iter<'a> = FilterMap<slice::Iter<'a, Option<Object>>, fn(&Option<Object>) -> Option<&Object>>;
}
use iter::{Enumerate, Iter};

pub struct Play {
    walls: &'static [Object],
    exits: room::Exits,
    score: u32,
    objects: Vec<Option<Object>>,
    facing: Side,
    player: Reference,
    motion: Displacement,
    on: On,
    now: Option<State>,
}

impl Room {
    pub fn start(&self, from: Entrance) -> Play {
        eprintln!("{}", self.name);
        for o in &self.objects {
        	eprintln!("{o:?}");
        }
        eprintln!("{:?}", self.environs);
        let mut objects = Vec::from_iter( 
            once(None)
            .chain(self.objects.iter().map(|object| (!object.is_cosmetic()).then(|| object.clone()) ))
        );
        self.objects.iter().filter_map(|host| host.effect().map(|spawn| Some(spawn))).collect_into(&mut objects);
        self.animate.as_ref()
        .map(|(count, kind)| from_fn(move || Some(kind.new())).take(count.get() as usize))
        .into_iter().flatten().collect_into(&mut objects);
                
        let mut this = Play {
            walls: &BOUNDS[self.walls()],
            exits: self.exits,
            score: 0,
            objects,
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

impl Object {
    fn effect(&self) -> Option<Object> {
        Some(match self.kind {
            Kind::Candle {..} => Object{
                kind: Kind::Flame,
                position: self.position - (3, 27),
            },
            Kind::Drip { range } => Object{
                kind: Kind::Drop(Motion::new(-7, (range as i16) << 5 + 1, 12)),
                position: self.position,
            },
            Kind::Fishbowl { range, delay } => Object {
                kind: Kind::Fish({let mut jump = Motion::new(-((range as i16) << 5), delay as i16, 12); jump.reset(); jump}),
                position: self.active_area()? * (Span::Center, Rise::Top) - (0, 2),

            },
            Kind::Toaster { range, delay } => Object {
                kind: Kind::Toast(
                    {let mut launch = Motion::new(-((range as i16) << 5), delay as i16, 12); launch.reset(); launch}, 
                    self.active_area()?.top() + 8
                ),
                position: self.position - (0, 18)
            },
            Kind::Teakettle { delay } => {
                let mut steam = Kind::Steam { progress: -10..(delay as i16) }.new()?;
                steam.position = (const{Size::new(41, 30).unwrap()} / (Span::Center, Rise::Bottom) << *self.position) * (Span::Left, Rise::Top);
                if let Kind::Steam{progress: Range{start, ..}} = &mut steam.kind {
                    *start += 20;
                    eprintln!("{steam:?}");
                    steam
                } else { return None }
            }
            _ => return None
        })
    }

    fn action(&self, mut test: Bounds, motion: &mut Displacement, id: object::Id, state: &Play) -> Option<Event> {
        use object::Kind;
        let previous = *motion;
        let (h, v) = motion.as_mut();
        match self.kind {
            Kind::CeilingDuct(Travel(None)) => Some(Event::Action(Change::Transport)),
            Kind::Exit{to: Some(room), ..} |
            Kind::CeilingDuct(Travel(Some(room))) => Some(Event::Control(State::Escaping(Some(room), 0..16))),
            Kind::CeilingDuct(Blow(..)) | Kind::CeilingVent {..} => {if state.on.air {*v = 8}; None},
            Kind::Fan { faces, .. } => {*h = faces * 7; (faces != state.facing).then_some(Event::Control(State::Turning(faces, 0..11))) }
            Kind::Steam{..} => {*motion.x_mut() -= 7; *motion.y_mut() -= 7; None}
            Kind::Grease {ready: true, ..} => Some(Event::Action(Change::Spill)),
            Kind::Grease {ready: false, ..} => {
                *motion.y_mut() -= motion.y().saturating_add(test.bottom()) - self.position.y();
                Some(Event::Control(State::Sliding(self.position.y())))
            }
            Kind::Shredder{..}
                => Some(Event::Control(State::Shredding{height: 0, x: self.position.x() + 3, top: self.position.y() + 2})),
            Kind::Table{..} | Kind::Shelf{..} | Kind::Books | Kind::Cabinet{..} | 
            Kind::Obstacle{..} | Kind::Basket | Kind::Macintosh 
                => Some(Event::Control(State::Landed)),
            Kind::Drop{..} | Kind::Toaster {..} | Kind::Ball{..} | Kind::Fishbowl {..} | 
            Kind::Fish{..} | Kind::Balloon(..) | Kind::Copter(..) | Kind::Dart(..)
                => Some(Event::Control(DIE)),
            Kind::Flame | Kind::Outlet{..} => Some(Event::Control(IGNITE)),
            Kind::Clock(..) | Kind::Bonus(..) |
            Kind::Battery(..) |
            Kind::Paper(..) |
            Kind::RubberBands(..) => Some(Event::Action(Change::Collect)),
            Kind::FloorVent { .. } | Kind::Candle { .. } => {if state.on.air {*v = -6}; None},
            Kind::Guitar => Some(Event::Display(Update::Start(Environment::Guitar, Some(id)))),
            Kind::Lights => Some(Event::Action(Change::Light)),
            Kind::Switch(target, _) => Some(Event::Action(Change::Toggle(target))),
            Kind::Thermostat => Some(Event::Action(Change::Heat)),
            Kind::Stair(flight, to) => Some(Event::Control(State::Stairs(flight, to))),
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
    Transport,
    Collect,
    Toggle(object::Id),
    Spill,
    Light,
    Heat,
}

enum Progress {
    Auto(Displacement, bool),
    Conclude(Outcome)
}

#[derive(Debug, Clone, Copy)]
pub enum Player {
    Flying{facing: Option<Side>, backward: bool},
    Shredding{height: u16}
}

impl Play {
    pub fn len(&self) -> usize { self.objects.len() }

    fn update(&mut self) -> Option<Progress> {
        let state = self.now.as_mut()?;
        if let State::Turning(faces, ..) = state { self.facing = *faces }
        if let Some(motion) = state.next() {
            let (motion, relative) = motion;
            let motion = if relative { motion * self.facing } else { motion };
            Some(Progress::Auto(motion, match state {State::Turning(..) |  State::Burning(..) => true, _ => false}))
        } else {
            self.now.take().and_then(|state| state.outcome(self.score))
            .map(|outcome|Progress::Conclude(outcome))
        }
    }
    fn apply(&mut self, source: object::Id, action: Change) -> Option<Update> {
        Some(match action {
            Change::Transport => {
                self.reset(Entrance::Air);
                Update::Start(Environment::Duct, None)
            }
            Change::Heat => {
                self.on.air = true;
                Update::Air
            }
            Change::Light => {
                self.on.lights = true;
                Update::Lights
            }
            Change::Spill => {
                match &mut self.objects[source.get()] {
                    Some(Object{kind: Kind::Grease{ready, ..}, ..}) if *ready == true => {*ready = false;}
                    _ => return None
                }
                Update::Start(Environment::Grease, Some(source))
            }
            Change::Toggle(id) => {
                if let Kind::Switch(_, Range{ref mut start, ..}) = self.objects[source.get()].as_mut()?.kind {
                    *start = -45;
                }
                match &mut self.objects[id.get()] {
                    Some(Object{kind: Kind::Shredder { ready }, ..}) |
                    Some(Object{kind: Kind::Fan { ready, .. }, ..}) 
                        => *ready = !*ready,
                    _ => ()
                };
                Update::Start(Environment::Switch, None)
            }
            Change::Collect => {
                return self.award(source)
            }
        })
    }

    pub fn frame(&mut self, actions: &[Input]) -> Outcome {
        let mut signal = self.now.as_ref().map(|s| match s {
            State::FadingIn(..) => vec![Update::Fade(true)],
            State::FadingOut(..) | State::Escaping(Some(_), ..) => vec![Update::Fade(false)],
            State::Burning(..) => vec![Update::Burn],
            State::Turning(..) => vec![Update::Turn(self.facing)],
            _ => vec![],
        });
        let control = match self.update() {
            Some(Progress::Conclude(outcome)) => return outcome,
            Some(Progress::Auto(motion, collision)) => Some((motion, collision)),
            _ => None
        };

        let (mut motion, collision) = control.unwrap_or_else(|| {
            let mut motion = Displacement::new(0, 3);
            for action in actions {
                match action {
                    Input::Go(direction) => *motion.x_mut() += *direction * MAX_THRUST,
                    Input::Flip => {
                        let turn = State::Turning(-self.facing, 0..11);
                        match &self.now {
                            Some(doing) if *doing >= turn => (),
                            _ => self.now = Some(turn)
                        };
                        signal.get_or_insert_with(|| vec![Update::Turn(-self.facing)]);
                    },
                    _ => ()
                };
            }
            (motion, true)
        });
        for animated in &mut self.objects {
            let Some(animated) = animated else {continue};
            animated.advance()
        }
        let events = if collision {
            if let Ok(touch) = Bounds::try_from(PLAYER_SIZE / (Span::Center, Rise::Center) << *self.player) {
                let objects = 
                    self.objects.iter().enumerate().filter_map(|(i, o)| Some((object::Id::try_from(i).ok()?, o.as_ref()?)) )
                        .chain(self.walls.iter().map(|wall| (const{object::Id(NonZero::new(usize::MAX).unwrap())}, wall)));
                let incidents = objects
                    .filter_map(|(i, o)| (o.active_area() & touch).and_then(|_| Some((i, o.action(touch, &mut motion, i, self)?))))
                    .collect::<Vec<_>>();

                let (events, outcomes) = incidents.into_iter().map(|(id, event)| 
                    match event {
                        Event::Display(update) => (Some(update), None),
                        Event::Control(state) => (None, Some(state)),
                        Event::Action(action) => (self.apply(id, action), None),
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

    fn award(&mut self, id: object::Id) -> Option<Update> {
        let object = self.objects[id.get()].take()?;
        let position = object.position;
        let ping = match object.kind {
            Kind::Battery(value) => {
                Update::Energy(value, position)
            }
            Kind::Bonus(value, _) |
            Kind::Clock(value) 
                => {
                    Update::Score(value, position)
                }
            Kind::Paper(value) => {
                Update::Life(value, position)
            }
            Kind::RubberBands(count) => {
                Update::Bands(count, position)
            }
            _ => return None
        };
        Some(ping)
    }

    pub fn dark(&self) -> bool { !self.on.lights }
    pub fn cold(&self) -> bool { !self.on.air }

    pub fn enumerate(&self) -> Enumerate<'_> {
        fn recode((index, option): (usize, &Option<Object>)) -> Option<(object::Id, &Object)> {
            Some((object::Id::try_from(index).ok()?, option.as_ref()?))
        }
        self.objects.iter().enumerate().filter_map(recode)
    }

    pub fn visible_items(&self) -> impl Iterator<Item = (crate::prelude::object::Id, &Object)> {
        self.enumerate()
            .filter_map(|(id, o)| { 
                if self.dark() {
                    match o.kind {
                        Kind::Lights | Kind::Balloon(..) | Kind::Dart(..) | Kind::Copter(..) | 
                        Kind::Outlet{progress: Range{start: ..=0, ..}, ..} | Kind::Drop(Motion{limit: Range{start: 1.., ..}, ..})
                            => (),
                        _ => return None,
                    }
                };
                Some((id.0, o))
            })
    } 

    pub fn player(&self) -> ((i16, i16), Player) {
        let backward = self.facing * self.motion.x() < 0;
        match self.now{
            Some(State::Shredding{height, x, top}) =>((x, top), Player::Shredding { height }),
            Some(State::Turning(..)) => ((self.player.into()), Player::Flying{facing: None, backward}),
            _ => ((self.player.into()), Player::Flying{facing: Some(self.facing), backward}),
        }
    }

    fn entrance(&self, from: Entrance) -> i16 {
        fn is_active_duct(o: &&Object) -> bool { matches!(o.kind, object::Kind::CeilingDuct { .. }) }
        fn is_down_stair(o: &&Object) -> bool { matches!(o.kind, object::Kind::Stair(Vertical::Down, _))}
        fn is_up_stair(o: &&Object) -> bool { matches!(o.kind, object::Kind::Stair(Vertical::Up, _))}
        self.into_iter()
        .filter(
            match from {
                Entrance::Air => is_active_duct,
                Entrance::Down => is_down_stair,
                Entrance::Up => is_up_stair,
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
            Entrance::Down => ((self.entrance(from) + 8, room::VERT_FLOOR as i16 - 10), Side::Right),
            Entrance::Up => ((self.entrance(from) + 8, room::VERT_CEILING as i16 + 10), Side::Right)
        }
    }
    pub fn reset(&mut self, at: Entrance) {
        let ((x, y), facing) = self.enter_at(at);
        if !matches!(at, Entrance::Air) {
			self.facing = facing;
        }
        self.player = Reference::new(x, y);
        self.motion = Displacement::default();
        self.now = at.action();
    }

    pub fn debug_zones<'this>(&'this self) -> impl Iterator<Item=Bounds> + 'this {
        self.objects.iter().filter_map(|o| o.as_ref()?.active_area())
    }
}

impl Index<object::Id> for Play {
    type Output = Object;
    fn index(&self, index: object::Id) -> &Self::Output {
        self.objects[index.get()].as_ref().expect("Accessed absent object::Id in Play.")
    }
}

impl IndexMut<object::Id> for Play {
    fn index_mut(&mut self, index: object::Id) -> &mut Self::Output {
        self.objects[index.get()].as_mut().expect("Accessed absent object::Id in Play.")
    }
}

impl<'a> IntoIterator for &'a Play {
    type IntoIter = Iter<'a>;
    type Item = &'a Object;
    fn into_iter(self) -> Self::IntoIter {
        self.objects.iter().filter_map(Option::as_ref)
    }
}