use crate::{room::{RoomId, Deactivated}, ObjectKind, Rect, Update};

use super::{Input, Outcome, room::{Room, Enemy}, Side, Object};
use std::{collections::BTreeSet, num::NonZeroU16, ops::{Deref, Range}};

const MAX_THRUST: i16 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    FadingIn(Range<u8>),
    FadingOut(Range<u8>),
    Turning(Range<u8>),
    Shredding(Rect),
    Burning(Range<u8>),
    Ascending(RoomId, u16),
    Descending(RoomId, u16),
}

impl State {
    fn outcome(&self, score: u32) -> Option<Outcome> {
        match self {
            Self::FadingOut(_) | Self::Burning(_) | Self::Shredding(_) => Some(Outcome::Dead),
            Self::Ascending(RoomId(room), _) | Self::Descending(RoomId(room), _) => Some(Outcome::Leave{score, destination: Some(*room)}),
            _ => None
        }
    }
}

impl std::iter::Iterator for State {
    type Item = (i16, i16);
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::FadingIn(phase) | 
            Self::FadingOut(phase) |
            Self::Burning(phase) |
            Self::Turning(phase) => phase.next().map(|_| (0, 0)),
            Self::Shredding(bounds) => match bounds.height().get() {
                0..36 => {bounds._bottom += 1; Some((0, (bounds.height().get() % 2) as i16))},
                _ => {bounds._top += 8; bounds._bottom += 8; (bounds._top > 342).then_some((0, 8))}
            },
            Self::Ascending(_, v) => {*v -= 6; (*v < 230).then_some((-2, -6))}
            Self::Descending(_, v) => {*v += 6; (*v > 130).then_some((2, 6))}
        }
    }
}

#[derive(Debug, Clone)]
enum Event {
    Control(State),
    Action(Update)
}

impl From<&State> for u8 {
    fn from(value: &State) -> Self {
        match value {
            State::Turning(phase) => 16u8 + phase.start,
            State::FadingIn(phase) => 32u8 + phase.start,
            State::FadingOut(phase) => 48u8 + phase.start,
            State::Shredding(_) => 64u8,
            State::Burning(_) => 80u8,
            State::Ascending(..) | State::Descending(..) => 96u8,
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
        u8::from(self).cmp(&u8::from(other))
    }
}

struct On {
    air: bool,
    lights: bool,
}

pub struct Play<'a> {
    room: &'a Room,
    score: u32,
    items: BTreeSet<usize>,
    facing: Side,
    player_h: u16,
    player_v: u16,
    motion_h: i16,
    motion_v: i16,
    on: On,
    now: Option<State>,
    pending: Option<Outcome>,
}

#[derive(Debug, Clone, Copy)]
pub enum Dynamic {
    Player{facing: Side, moving: Option<Side>, bounds: Rect},
    Enemy{facing: Side, kind: Enemy, bounds: Rect},
    Grease{facing: Side, spill: Option<NonZeroU16>, bounds: Rect},
}

#[derive(Debug, Clone, Copy)]
pub enum Entrance {
    Flying(Side),
    Appearing(u8),
}

impl Room {
    pub fn collider_ids(&self) -> impl Iterator<Item = usize> + '_ {
        self.objects.iter().enumerate().filter_map(|(id, o)| o.collidable().then_some(id))
    }
    pub fn start(&self, from: Entrance, _lights: bool, _air: bool) -> Play {
        let (x, y, facing) = match from {
            Entrance::Flying(side) => (match side { Side::Left => 24, Side::Right => 488}, 50, -side),
            Entrance::Appearing(target) => {let bounds = self.objects[target as usize].bounds; (bounds.x(), bounds.y(), Side::Right)}
        };
        Play { 
            room: self,
            score: 0,
            items: BTreeSet::<usize>::from_iter(self.collider_ids()),
            facing: facing,
            player_h: x,
            player_v: y,
            motion_h: 0,
            motion_v: 0,
            on: On{air: self.condition_code != Some(Deactivated::Air), lights: self.condition_code != Some(Deactivated::Lights)},
            now: None,
            pending: None,
        }
    }
}

impl super::object::Object {
    fn action(&self, id: usize) -> Option<Box<dyn Fn(&mut Play, &mut(i16, i16)) -> Option<Event>>> {
        type Kind = ObjectKind;
        Some(match (self.object_is, self.is_on) {
            (Kind::CeilingDuct { destination, .. }, true) => {let destination = destination.into(); Box::new(move |play, _| {play.cue(Outcome::Leave{score: play.score, destination}); None})},
            (Kind::CeilingDuct {..}, false) | (Kind::CeilingVent {..}, _) => Box::new(|play, (_, v)| {if play.on.air {*v += 7}; None}),
            (Kind::Fan { faces, .. }, true) => Box::new(move |play, (h, _)| {*h += faces * 7; let flip = faces != play.facing; play.facing = faces; flip.then_some(Event::Control(State::Turning(0..11)))}),
            (kind, _) => match kind {
                Kind::Table | Kind::Shelf | Kind::Books | Kind::Cabinet | Kind::Obstacle | Kind::Basket | Kind::Macintosh |
                Kind::Drip{..} | Kind::Toaster {..} | Kind::Ball{..} | Kind::Fishbowl {..} => Box::new(|play, _| {
                    play.cue(Outcome::Dead);
                    Some(Event::Control(State::FadingOut(0..16)))
                }),
                Kind::Clock(value) | Kind::Bonus(value) => Box::new(move |play, _| {play.award(value); play.items.remove(&id); None}),
                Kind::FloorVent { .. } => Box::new(|play, (_, v)| {if play.on.air {*v -= 7}; None}),
                _ => return None
            }
        })
    }
}

impl<'a> Play<'a> {
    pub fn frame(&mut self, actions: &[Input]) -> Outcome {
        let control = if let Some(state) = self.now.as_mut() {
            if let Some(motion) = state.next() {
                Some((motion, match state {State::Turning(_) | State::Burning(_) => true, _ => false}))
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
            let touch = Rect::cropped_on((self.player_h, self.player_v), 28, 10);
            let actions: Vec<_> = self.active_items().enumerate().filter_map(|(i, o)| (o.active_area() & touch).and_then(|_| o.action(i))).collect();
            let (events, outcomes): (Vec<_>, Vec<_>) = actions.iter().filter_map(|f| f(self, &mut motion).map(|e| {
                match e {
                    Event::Action(a) => (Some(a), None),
                    Event::Control(c) => (None, Some(c)),
                }
            })).unzip();
            let events: Vec<_> = events.into_iter().filter_map(|e| e).collect();
            self.now = self.now.iter().chain(outcomes.iter().filter_map(|e| e.as_ref())).max().cloned();
            Some(events)
        } else {
            None
        };
        self.motion_h = motion.0;
        self.motion_v = motion.1;
        self.player_h = self.player_h.saturating_add_signed(motion.0);
        self.player_v = self.player_v.saturating_add_signed(motion.1);
        Outcome::Continue(events)
    }

    fn cue(&mut self, what: Outcome) {
        self.pending = match what {
            Outcome::Continue(_) => match &self.pending {
                None => Some(what.clone()),
                o => o.clone(),
            },
            Outcome::Dead => match &self.pending {
                None | Some(Outcome::Continue(_)) => Some(Outcome::Dead),
                o => o.clone()
            }
            leave => Some(leave),
        };
    }
    fn award(&mut self, value: u16) {
        self.score += value as u32;
    }

    pub fn active_items(&self) -> impl Iterator<Item = &Object> {
        self.items.iter()
            .map(|&index| &self.room.objects[index] )
    }

    pub fn player(&self) -> ((u16, u16), Side, bool) {
        ((self.player_h, self.player_v), self.facing, self.facing * self.motion_h < 0)
    }
}