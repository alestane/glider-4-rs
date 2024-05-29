use crate::Rect;

use super::{Input, Outcome, room::{Room, Enemy}, Side, Object};
use std::{collections::BTreeSet, num::NonZeroU16};

pub struct Play<'a> {
    room: &'a Room,
    items: BTreeSet<usize>,
    facing: Side,
    player_h: u16,
    player_v: u16,
}

pub enum Dynamic {
    Player{facing: Side, moving: Option<Side>, bounds: Rect},
    Enemy{facing: Side, kind: Enemy, bounds: Rect},
    Grease{facing: Side, spill: Option<NonZeroU16>, bounds: Rect},
}

pub enum Entrance {
    Flying(Side),
    Appearing(u8),
}

impl Room {
    pub fn collider_ids(&self) -> impl Iterator<Item = usize> + '_ {
        self.objects.iter().enumerate().filter_map(|(id, o)| o.collidable().then_some(id))
    }
    pub fn start(&self, from: Entrance, _lights: bool, _air: bool) -> Play {
        let (x, y) = match from {
            Entrance::Flying(side) => (match side { Side::Left => 24, Side::Right => 488}, 50),
            Entrance::Appearing(target) => {let bounds = self.objects[target as usize].bounds; (bounds.x(), bounds.y())}
        };
        Play { 
            room: self,
            items: BTreeSet::<usize>::from_iter(self.collider_ids()),
            facing: Side::Right,
            player_h: x,
            player_v: y,
        }
    }
}

impl<'a> Play<'a> {
    pub fn frame(&mut self, _actions: &[Input]) -> Outcome {
        Outcome::Continue
    }

    pub fn active_items(&self) -> impl Iterator<Item = &Object> {
        self.items.iter()
            .map(|&index| &self.room.objects[index] )
    }

    pub fn player(&self) -> ((u16, u16), Side) {
        ((self.player_h, self.player_v), self.facing)
    }
}