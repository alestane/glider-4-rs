use crate::{ObjectKind, Rect, Update};

use super::{Input, Outcome, room::{Deactivated, Room, Enemy}, Side, Object};
use std::{collections::{BTreeSet, HashMap}, num::NonZero, ops::Range};

const MAX_THRUST: i16 = 5;

#[derive(Debug, Clone)]
struct Hazard {
    kind: Enemy,
    period: Range<u16>,
    bounds: Rect,
    is_on: bool
}

impl From<Object> for Option<Hazard> {
    fn from(value: Object) -> Self {
        let bounds = value.bounds;
        Option::<Enemy>::from(value.object_is).and_then(|kind|
            Some(match kind {
                Enemy::Flame => Hazard{kind, period: 0..0, bounds: Rect::new(bounds.left() + 5, bounds.top() - 12, bounds.left() + 16, bounds.top()), is_on: true},

                _ => return None
            })
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    FadingIn(Range<u8>),
    FadingOut(Range<u8>),
//    Turning(Range<u8>),
//    Shredding(Rect),
    Burning(Range<u16>),
//    Ascending(RoomId, u16),
//   Descending(RoomId, u16),
}

const DIE: State = State::FadingOut(0..16);
const IGNITE: State = State::Burning(0..150);

impl State {
    fn outcome(&self, _score: u32) -> Option<Outcome> {
        match self {
            Self::FadingOut(_) | Self::Burning(_) /* | Self::Shredding(_) */ => Some(Outcome::Dead),
//            Self::Ascending(RoomId(room), _) | Self::Descending(RoomId(room), _) => Some(Outcome::Leave{score, destination: Some(*room)}),
            _ => None
        }
    }
}

impl std::iter::Iterator for State {
    type Item = (i16, i16, bool);
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::FadingIn(phase) |
            Self::FadingOut(phase) /* |
            Self::Turning(phase) */ => phase.next().map(|_| (0, 0, false)),
            Self::Burning(phase) => {if phase.next().is_none() {eprintln!("burn timeout"); *self = DIE}; Some((1, 3, true)) },
            /* Self::Shredding(bounds) => match bounds.height().get() {
                0..36 => {bounds._bottom += 1; Some((0, (bounds.height().get() % 2) as i16))},
                _ => {bounds._top += 8; bounds._bottom += 8; (bounds._top > 342).then_some((0, 8))}
            },
            Self::Ascending(_, v) => {*v -= 6; (*v < 230).then_some((-2, -6))}
            Self::Descending(_, v) => {*v += 6; (*v > 130).then_some((2, 6))} */
        }
    }
}

#[derive(Debug, Clone)]
enum Event {
    Control(State),
    Action(Update, Option<usize>),
}

impl From<&State> for u8 {
    fn from(value: &State) -> Self {
        match value {
        //    State::Turning(phase) => 16u8 + phase.start,
            State::FadingIn(phase) => 32u8 + phase.start,
            State::FadingOut(phase) => 48u8 + phase.start,
        //    State::Shredding(_) => 64u8,
            State::Burning(_) => 80u8,
        //    State::Ascending(..) | State::Descending(..) => 96u8,
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

struct On {
    air: bool,
    lights: bool,
}

fn id() -> u8 {
    static mut NEXT: NonZero<u8> = unsafe { NonZero::new_unchecked(73) };
    let id = unsafe { NEXT.get() };
    unsafe { NEXT = NonZero::new(id + 73).unwrap_or(NonZero::new_unchecked(73) ) };
    id
}

pub struct Play<'a> {
    room: &'a Room,
    score: u32,
    items: BTreeSet<usize>,
    facing: Side,
    player_h: i16,
    player_v: i16,
    motion_h: i16,
    motion_v: i16,
    on: On,
    now: Option<State>,
    hazards: HashMap<u8, Hazard>,
}

#[derive(Debug, Clone, Copy)]
pub enum Dynamic {
    Player{facing: Side, moving: Option<Side>, bounds: Rect},
    Enemy{facing: Side, kind: Enemy, bounds: Rect},
    Grease{facing: Side, spill: Option<NonZero<u16>>, bounds: Rect},
}

#[derive(Debug, Clone, Copy)]
pub enum Entrance {
    Spawn(Side),
    Flying(Side, u16),
//    Up, Down, Air,
}

impl Default for Entrance {
    fn default() -> Self { Self::Spawn(Side::Left) }
}

impl Entrance {
    fn action(&self) -> Option<State> {
        match self {
            Self::Spawn(..) => Some(State::FadingIn(0..16)),
            _ => None
        }
    }
}

impl Room {
    pub fn collider_ids(&self) -> impl Iterator<Item = usize> + '_ {
        self.objects.iter().enumerate().filter_map(|(id, o)| o.collidable().then_some(id))
    }
    fn enter_at(&self, from: Entrance) -> ((i16, i16), Side) {
        match from {
            Entrance::Spawn(side) => ((match side { Side::Left => 24, Side::Right => 488}, 50), -side),
            Entrance::Flying(side, height) => ((match side { Side::Left => 24, Side::Right => 488}, height as i16), -side),
//            Entrance::Appearing(target) => {let bounds = self.objects[target as usize].bounds; (bounds.x(), bounds.y(), Side::Right)}
        }
    }
    pub fn start(&self, from: Entrance) -> Play {
        let ((x, y), facing) = self.enter_at(from);
        Play {
            room: self,
            score: 0,
            items: BTreeSet::<usize>::from_iter(self.collider_ids()),
            facing,
            player_h: x,
            player_v: y,
            motion_h: 0,
            motion_v: 0,
            on: On{air: self.condition_code != Some(Deactivated::Air), lights: self.condition_code != Some(Deactivated::Lights)},
            now: from.action(),
            hazards: HashMap::from_iter(self.objects.iter().filter_map(|o| Option::<Hazard>::from(*o)).map(|h| (id(), h))),
        }
    }
}

impl super::object::Object {
    fn action(&self, mut test: Rect, motion: &mut(i16, i16), id: usize, air: bool) -> Option<Event> {
        type Kind = ObjectKind;
        match (self.object_is, self.is_on) {
//            (Kind::CeilingDuct { destination, .. }, true) => {let destination = destination.into(); Box::new(move |play, _| {play.cue(Outcome::Leave{score: play.score, destination}); None})},
//            (Kind::CeilingDuct {..}, false) | (Kind::CeilingVent {..}, _) => Box::new(|play, (_, v)| {if play.on.air {*v += 7}; None}),
//            (Kind::Fan { faces, .. }, true) => Box::new(move |play, (h, _)| {*h += faces * 7; let flip = faces != play.facing; play.facing = faces; flip.then_some(Event::Control(State::Turning(0..11)))}),
            (kind, _) => match kind {
                Kind::Table | Kind::Shelf | Kind::Books | Kind::Cabinet | Kind::Obstacle | Kind::Basket | Kind::Macintosh |
                Kind::Drip{..} | Kind::Toaster {..} | Kind::Ball{..} | Kind::Fishbowl {..} => Some(Event::Control(DIE)),
                Kind::Clock(value) | Kind::Bonus(value) => Some(Event::Action(Update::Score(value), Some(id))),
                Kind::FloorVent { .. } | Kind::Candle { .. } => {if air {motion.1 -= 7}; None},
                Kind::Wall => {
                    test >>= *motion;
                    if test.left() < self.bounds.right() && test.right() >= self.bounds.right() {
                        motion.0 += (self.bounds.right() - test.left()) as i16;
                    }
                    if test.right() > self.bounds.left() && test.left() <= self.bounds.left() {
                        motion.0 -= (self.bounds.left() - test.right()) as i16;
                    }
                    Some(Event::Action(Update::Bump, None))
                }
                _ => None
            }
        }
    }
}

const BOUNDS: [Object; 3] = [
    Object{
        object_is: ObjectKind::Wall,
        bounds: Rect::new(0, 0, 14, 342),
        is_on: true,
    },
    Object{
        object_is: ObjectKind::Obstacle,
        bounds: Rect::new(0, 325, 512, 342),
        is_on: true,
    },
    Object{
        object_is: ObjectKind::Wall,
        bounds: Rect::new(512, 0, 536, 342),
        is_on: true,
    },
];

impl<'a> Play<'a> {
    pub fn frame(&mut self, actions: &[Input]) -> Outcome {
        let signal = self.now.as_ref().map(|s| match s {
            State::FadingIn(..) => vec![Update::Fade(true)],
            State::FadingOut(..) => vec![Update::Fade(false)],
            State::Burning(..) => vec![Update::Burn],
        });
        let control = if let Some(state) = self.now.as_mut() {
            if let Some(motion) = state.next() {
            	let (h, v, relative) = motion;
            	let motion = if relative { (self.facing * h, v) } else { (h, v) };
                Some((motion, match state {/* State::Turning(_) | */ State::Burning(..) => true, _ => false}))
            } else {
                if let Some(outcome) = state.outcome(self.score) {
                    return outcome
                }
                self.now = None;
                None
            }
        } else { None };

        let (mut motion, collision) = if let Some(o) = control {
            o
        } else {
            let mut motion = (0, 3);
            for action in actions {
                match action {
                    Input::Go(direction) => motion.0 += *direction * MAX_THRUST,
                    _ => ()
                };
            }
            (motion, true)
        };
        let events = if collision {
            let walls = &BOUNDS[self.room.walls()];
            let touch = Rect::cropped_on((0u16.saturating_add_signed(self.player_h), 0u16.saturating_add_signed(self.player_v)), 28, 10);
            let actions: Vec<_> = self.active_items().chain(walls).enumerate().filter_map(|(i, o)|
            	(o.active_area() & touch).and_then(|_| o.action(touch, &mut motion, i, self.on.air))
            ).chain(self.hazards.values().filter_map(|h|
                h.is_on
                .then_some(h.bounds)
                .and_then(|bounds| (bounds & touch).map(|_| Event::Control(match h.kind {Enemy::Flame | Enemy::Shock => IGNITE, _ => DIE})))
            )).collect();
            let (events, outcomes): (Vec<_>, Vec<_>) = actions.into_iter().map(|e| {
                match e {
                    Event::Action(a, remove) => {
                        if let Some(ref used) = remove {self.items.remove(used);};
                        (Some(a), None)
                    },
                    Event::Control(c) => (None, Some(c)),
                }
            }).unzip();

            let events: Vec<_> = signal.into_iter().flatten().chain(events.into_iter().filter_map(|e| e)).collect();
            self.now = self.now.iter().chain(outcomes.iter().filter_map(|e| e.as_ref())).max().cloned();
            Some(events)
        } else {
            signal
        };
        self.motion_h = motion.0;
        self.motion_v = motion.1;
        self.player_h = self.player_h + motion.0;
        self.player_v = self.player_v + motion.1;
        if let Some(out) = match self.player_h {..-12 => Some(Side::Left), 489.. => Some(Side::Right), _ => None} {
            return Outcome::Leave{score: self.score, destination: Some((out * 1, Entrance::Flying(-out, self.player_v as u16)))}
        };
        Outcome::Continue(events)
    }

    fn award(&mut self, value: u16) {
        self.score += value as u32;
    }

    pub fn active_items(&self) -> impl Iterator<Item = &Object> {
        self.items.iter()
            .map(|&index| &self.room.objects[index] )
    }

    pub fn active_hazards(&self) -> impl Iterator<Item = (u8, Enemy, Rect)> + '_ {
        self.hazards.iter().map(|(&id, Hazard{kind, bounds, ..})| (id, *kind, *bounds))
    }

    pub fn player(&self) -> ((i16, i16), Side, bool) {
        ((self.player_h, self.player_v), self.facing, self.facing * self.motion_h < 0)
    }

    pub fn reset(&mut self, at: Entrance) {
        let ((x, y), facing) = self.room.enter_at(at);
        self.facing = facing;
        self.player_h = x;
        self.player_v = y;
        self.motion_h = 0;
        self.motion_v = 0;
        self.now = Some(State::FadingIn(0..16));
    }
}
