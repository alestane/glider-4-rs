use std::ops::Range;

use crate::{object, room, Displacement, Interval, Object, Position};

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
            if self.limit.next().is_none() { self.limit.start = 0; return None }
            return Some(0);
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
                    let reset = motion.limit.start >> 5;
                    motion.limit.start = -7;
                    (0, -reset)
                } else {
                    let position = motion.limit.start >> 5;
                    motion.velocity += motion.acceleration;
                    motion.limit.start += motion.velocity;
                    (0, (motion.limit.start >> 5) - position)
                }
            }
            Is::Ball(motion) | Is::Toast(motion, ..) | Is::Fish(motion) => {
                (0, if let Some(v) = motion.next() {v} else {motion.velocity = -motion.velocity; 0})
            }
            Is::Balloon(delay) => delay.next().is_none().then_some( (0, -3) )?, 
            Is::Copter(delay) =>  delay.next().is_none().then_some( (-4, 2) )?,
            Is::Dart(delay) =>    delay.next().is_none().then_some( (-8, 1) )?,
            Is::Grease { ready: false, progress } => {progress.next(); return None},
            Is::Outlet { progress } => {if let None = progress.next() {progress.start = -30;} return None},
            Is::Steam { progress } => {if let None = progress.next() {progress.start = -10;} return None},
            _ => return None,
            
        }.into())
    }
}

impl Object {
    pub fn advance(&mut self) {
        if let Some(offset) = self.kind.next() {
            type Is = object::Kind;
            self.position += <(_,_)>::from(offset);
            match self.kind {
                Is::Balloon(..) | Is::Copter(..) | Is::Dart(..) if (self.active_area() & room::BOUNDS).is_none() 
                    => self.reset(),
                _ => ()
            };
        }
    }

    fn reset(&mut self) {
        if let Some(position) = self.kind.reset(0) {
            self.position = position;
        }
    }
}

impl object::Kind {
    fn reset(&mut self, delay: i16) -> Option<Position> {
        type Is = object::Kind;
        let (start, position) = match self {
            Is::Balloon(Range{start, ..}) => (start, (random() % 400 + 50, 358)),
            Is::Copter(Range{start, ..}) => (start, (random() % 256 + 272, -16)),
            Is::Dart(Range{start, ..}) => (start, (544, random() % 150 + 11)),
            _ => return None,
        };
        *start = -delay;
        Some(position.into())
    }

    pub fn new(&self) -> Option<Object> {
        let mut kind = self.clone();
        let position = kind.reset(random() % 60 + 30)?;
        Some(Object{kind, position})
    }
}

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