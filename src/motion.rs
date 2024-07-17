use crate::{object, Displacement, Interval, Object};

#[disclose]
#[derive(Debug, Clone)]
pub struct Motion<const N: usize = 0> {
    limit: Interval,
    acceleration: i16,
    velocity: i16,
}

impl Motion {
    pub fn new(start: i16, limit: i16, acceleration: i16) -> Self {
        Self { limit: start..limit, acceleration, velocity: 0 }
    }
    pub fn reset(&mut self) {
        self.velocity = 0;
        while self.limit.start < 0 {
            self.velocity -= self.acceleration;
            self.limit.start -= self.velocity;
        }
        self.limit.start = 0;
    }
    pub fn value(&self) -> i16 { self.limit.start }
}

impl Iterator for Motion {
    type Item = i16;
    fn next(&mut self) -> Option<Self::Item> {
        if self.limit.start > 0 { 
            if self.limit.next().is_none() { self.limit.start = 0; }
            return None
        }
        let position = self.limit.start >> 5;
        self.velocity += self.acceleration;
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
                motion.velocity += motion.acceleration;
                motion.limit.start += motion.velocity;
                (0, (motion.limit.start >> 5) - position)
            }
            Is::Ball(motion) | Is::Toast(motion) | Is::Fish(motion) => {
                (0, motion.next()?)
            }
            Is::Balloon(delay) => { delay.next()?; (0, -3) }, 
            Is::Copter(delay) =>  { delay.next()?; (-8, 1) },
            Is::Dart(delay) =>    { delay.next()?; (-4, 2) },
            Is::Spill { progress } => {progress.next(); return None},
            Is::Outlet { progress } => {if let None = progress.next() {progress.start = -30;} return None},
            Is::Steam { progress } => {if let None = progress.next() {progress.start = -10;} return None},
            _ => return None,
            
        }.into())
    }
}

impl Object {
    pub fn advance(&mut self) {
        if let Some(offset) = self.kind.next() {self.position += <(_,_)>::from(offset)}
    }
}