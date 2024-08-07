use sdl2::{keyboard::{KeyboardState, Scancode}, surface::Surface};
use glider::{Entrance, Environment, Input, Outcome, Play, Room, Side, Update};
use crate::{atlas, draw::{Animations, Frame, Scribe}, object, room::{self}};
use std::{collections::HashMap, error::Error, fmt::Display, iter::repeat, num::NonZero, ops::Range, time::{Duration, Instant}};

const FADE_IN: &[usize] = &[3, 4, 3, 4, 5, 4, 5, 6, 5, 6, 7, 6, 7, 8, 7, 8, 9];
const FADE_OUT: &[usize] = &[9, 8, 9, 8, 7, 8, 7, 6, 7, 6, 5, 6, 5, 4, 5, 4, 3];

pub struct Game {
    score: u32,
    current_room: room::Id,
    rooms: HashMap<room::Id, (Play, Surface<'static>)>,
}

fn animate_with<F: FnOnce() -> Frame>(list: &Animations, id: usize, loader: F) {
	let mut list = list.borrow_mut();
    if !list.contains_key(&id) {list.insert(id, loader());}
}

fn recycle<I: Iterator<Item: Clone>>(iter: I, count: usize) -> impl Iterator<Item = I::Item> {
    iter.flat_map(move |i| repeat(i).take(count))
}

#[derive(Debug)]
enum PlayRoomError {
    UnknownRoom(room::Id),
    PlayerQuit,
}

impl Display for PlayRoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlayerQuit => write!(f, "Player closed game"),
            #[cfg(debug_assertions)]
            Self::UnknownRoom(id) => write!(f, "Left house for pending room #{}", id.get()),
            #[cfg(not(debug_assertions))]
            Self::UnknownRoom(id) => write!(f, "Room #{} doesn't exist in this game", id.get()),
        }
    }
}

impl Error for PlayRoomError {}

impl Game {
    pub fn run(&mut self, context: &mut crate::App, target: Entrance) -> Result<(u32, Option<(NonZero<u16>, Entrance)>), Box<dyn Error>> {
        let room = self.current_room;
        let display = &mut context.display;
        display.set_blend_mode(sdl2::render::BlendMode::Blend);
        let creator = display.texture_creator();
        let sprites = atlas::glider_sprites(context.sprites.as_ref().as_texture(&creator)?);

        let Some((play, wall)) = self.rooms.get_mut(&room) else { return Err(Box::new(PlayRoomError::UnknownRoom(room))) };

        let wall = wall.as_texture(&creator)?;
        let mut display = (display, &sprites);

        play.reset(target);

        let animation = Animations::default();
        {
            let mut animation = animation.borrow_mut();
            for (id, object) in play.enumerate() {
                let range = match object.kind {
                    object::Kind::Balloon(Range{end, ..}) => (end as usize, atlas::RISING.count()),
                    object::Kind::Copter(Range{end, ..}) => (end as usize, atlas::FALLING.count()),
                    object::Kind::Dart(..) => {
                        animation.insert(id.get(), Box::new(repeat(1)));
                        continue
                    }
                    object::Kind::Flame => (atlas::FLAME.start, atlas::FLAME.end),
                    _ => continue,
                };
                animation.insert(id.get(), Box::new(recycle((range.0..range.1).cycle(), 2)));
            }
        }

        let mut last = Instant::now();
        'game: loop {
            while last.elapsed() < Duration::from_millis(33) {}
            let mut inputs = Vec::new();
            let keys = KeyboardState::new(&context.events);
            let left = keys.is_scancode_pressed(Scancode::Left);
            let right = keys.is_scancode_pressed(Scancode::Right);
            if right {inputs.push(Input::Go(Side::Right))};
            if left {inputs.push(Input::Go(Side::Left))};
            for event in context.events.poll_iter() {
                use sdl2::event::Event;
                match event {
                    Event::Quit{..} => break 'game,
                    Event::KeyDown { scancode: Some(Scancode::Up), repeat: false, .. } => inputs.push(Input::Flip),
                    Event::KeyDown { scancode: Some(Scancode::Escape), repeat: false, .. } => break 'game,
                    _ => ()
                }
            }

            let result = play.frame(&inputs);

            match result {
                Outcome::Continue(updates) => {
                    for update in updates.into_iter().flatten() {
                        match update {
                            Update::Turn(side) => animate_with(&animation, 0, || match side {Side::Right => Box::new(recycle((0..6).rev(), 2)), Side::Left => Box::new(recycle(0..6, 2))}),
                            Update::Fade(inout) => animate_with(&animation, 0, || if inout {Box::new(FADE_IN.iter().cloned())} else {Box::new(FADE_OUT.iter().cloned())}),
                            Update::Burn => animate_with(&animation, 0, || Box::new(atlas::BURN.cycle()) ),
                            Update::Start(Environment::Grease, Some(bottle)) => animate_with(&animation, bottle.get(), 
                                || Box::new(recycle(atlas::TIPPING..=atlas::TIPPING, 2))
                            ),
                            _ => ()
                        }
                    }
                    display.show(&sdl2::pixels::Color::RGB(0, 0, 0));
                    if !play.dark() {display.show(&wall);}
                    display.show(&(&*play, &animation));
                },
                Outcome::Dead => {
                    animation.borrow_mut().remove(&0);
                    play.reset(match target {Entrance::Flying(side, ..) => Entrance::Spawn(side), target => target})
                }
                Outcome::Leave{destination: Some((to_room, at)), ..} if to_room == room => play.reset(at),
                Outcome::Leave{score, destination} => return Ok((score, destination)),
                _ => ()
            }
            last = Instant::now();
        }
        Err(Box::new(PlayRoomError::PlayerQuit))
    }

    pub fn play(&mut self, context: &mut crate::App) -> Result<(u32, NonZero<u16>), Box<dyn Error>> {
        let mut arrive = Entrance::default();
        while let (points, Some((next, at))) = {
            let (room, _) = self.rooms.get_mut(&self.current_room).ok_or(PlayRoomError::UnknownRoom(self.current_room))?;
            eprintln!("Object count: {}", room.len());
            self.run(context, arrive)?
        } {
            self.score += points;
            if next.get() as usize > self.len() { return Err(Box::new(PlayRoomError::UnknownRoom(next))) }
            (self.current_room, arrive) = (next, at);
        } 
        eprintln!("Left house to {:?}", self.current_room);
        Ok((self.score, self.current_room))
    }
    fn len(&self) -> usize { self.rooms.len() }
}

impl crate::App {
    pub fn prepare(&mut self, house: &[Room], themes: &HashMap<usize, Surface>) -> Result<Game, Box<dyn Error>> {
        Ok(Game{
            score: 0,
            #[cfg(not(debug_assertions))]
            current_room: const{ NonZero::new(1).unwrap() },
            #[cfg(debug_assertions)]
            current_room: crate::test::START,
            rooms: house.iter().enumerate().map(|(i, r)| 
                Ok::<_, Box<dyn Error>> ((
                    NonZero::new(i as u16 + 1).unwrap(), 
                    (
                        r.start(Entrance::default()), 
                        {
                            let mut room = Surface::new(room::SCREEN_WIDTH, room::SCREEN_HEIGHT, self.display.default_pixel_format())?.into_canvas()?;
                            let processor = room.texture_creator();
                            let sprites = atlas::glider_sprites(self.sprites.as_ref().as_texture(&processor)?);
                            (&mut room, &sprites).show(&(&themes[&(r.theme_index() as usize)], r));
                            room.into_surface()
                        }
                    )
                ))
            )
            .try_collect()?
        })
    }
}
