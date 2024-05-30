use crate::Rect;

use super::{Input, Outcome, room::{Room, Enemy}, Side, Object};
use std::{collections::BTreeSet, num::NonZeroU16};

const MAX_THRUST: i16 = 5;

pub struct Play<'a> {
    room: &'a Room,
    items: BTreeSet<usize>,
    facing: Side,
    player_h: u16,
    player_v: u16,
    motion_h: i16,
    motion_v: i16,
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
            items: BTreeSet::<usize>::from_iter(self.collider_ids()),
            facing: facing,
            player_h: x,
            player_v: y,
            motion_h: 0,
            motion_v: 0,
        }
    }
}

impl<'a> Play<'a> {
    pub fn frame(&mut self, actions: &[Input]) -> Outcome {
        let mut motion = (0i16, 0i16);
        for action in actions {
            match action {
                Input::Go(direction) => motion.0 += *direction * MAX_THRUST,
                _ => ()
            };
        }
        self.motion_h = motion.0;
        self.player_h = self.player_h.saturating_add_signed(motion.0);
        Outcome::Continue
    }

    pub fn active_items(&self) -> impl Iterator<Item = &Object> {
        self.items.iter()
            .map(|&index| &self.room.objects[index] )
    }

    pub fn player(&self) -> ((u16, u16), Side, bool) {
        ((self.player_h, self.player_v), self.facing, self.facing * self.motion_h < 0)
    }
}