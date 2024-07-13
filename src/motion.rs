use std::ops::Range;
use crate::{object, Displacement};

#[disclose]
#[derive(Debug, Clone)]
struct Motion<const N: usize = 0> {
    limit: Range<i16>,
    accel: i16,
    velocity: i16,
}

impl Motion {
    pub fn reset(&mut self) {
        self.velocity = 0;
        while self.limit.start < 0 {
            self.velocity -= self.accel;
            self.limit.start -= self.velocity;
        }
        self.limit.start = 0;
    }
}

impl Iterator for Motion {
    type Item = i16;
    fn next(&mut self) -> Option<Self::Item> {
        if self.limit.start > 0 { 
            if self.limit.next().is_none() { self.limit.start = 0; }
            return None
        }
        let position = self.limit.start >> 5;
        self.velocity += self.accel;
        self.limit.start = 1.min(self.limit.start + self.velocity);
        Some((self.limit.start >> 5) - position)
    }
}

impl Iterator for object::Kind {
    type Item = Displacement;
    fn next(&mut self) -> Option<Self::Item> {
        type Is = object::Kind;
        Some(match self {
            Is::Drop(motion) => {
                if motion.limit.start <= 0 {
                    motion.limit.start += 1;
                    return None
                }
                if motion.limit.start >= motion.limit.end { 
                    motion.velocity = 0;
                    motion.limit.start = -8;
                    return None 
                }
                let position = motion.limit.start >> 5;
                motion.velocity += motion.accel;
                motion.limit.start += motion.velocity;
                (0, (motion.limit.start >> 5) - position)
            }
            Is::Ball(motion) | Is::Toast(motion) | Is::Fish(motion) => {
                (0, motion.next()?)
            }
            Is::Balloon => (0, -3),
            Is::Dart =>  (-8, 1),
            Is::Copter => (-4, 2),
            Is::Spill { progress, ready } => {if *ready {progress.next();} return None},
            Is::Shock { progress } => {if let None = progress.next() {progress.start = -30;} return None},
            Is::Steam { progress } => {if let None = progress.next() {progress.start = -10;} return None},
            _ => return None,
            
        }.into())
    }
}

