use sdl2::{keyboard::{KeyboardState, Scancode}, render::Texture};
use glider::{Entrance, Environment, Input, Outcome, Room, Side, Update};
use crate::{atlas, draw::{Animations, Frame, Scribe}, room::{SCREEN_HEIGHT, SCREEN_WIDTH}, object};
use std::{iter::repeat, num::NonZero, ops::Range, time::{Duration, Instant}};

const FADE_IN: &[usize] = &[3, 4, 3, 4, 5, 4, 5, 6, 5, 6, 7, 6, 7, 8, 7, 8, 9];
const FADE_OUT: &[usize] = &[9, 8, 9, 8, 7, 8, 7, 6, 7, 6, 5, 6, 5, 4, 5, 4, 3];

fn animate_with<F: FnOnce() -> Frame>(list: &Animations, id: usize, loader: F) {
	let mut list = list.borrow_mut();
    if !list.contains_key(&id) {list.insert(id, loader());}
}

fn recycle<I: Iterator<Item: Clone>>(iter: I, count: usize) -> impl Iterator<Item = I::Item> {
    iter.flat_map(move |i| repeat(i).take(count))
}

pub fn run(context: &mut crate::App, theme: &Texture, room: (NonZero<u16>, &Room), target: Entrance) -> Result<(u32, Option<(NonZero<u16>, Entrance)>), ()> {
    let display = &mut context.display;
    let loader = display.texture_creator();

    let mut backdrop = loader.create_texture_target(None, SCREEN_WIDTH, SCREEN_HEIGHT).expect("Failed to create backdrop texture");
    let _ = display.with_texture_canvas(&mut backdrop,
        |display| {
            let mut display = (display, &context.sprites);
            display.show(&(theme, room.1));
        }
    );
    let mut display = (display, &context.sprites);
    let mut play = room.1.start(target);
    if let Entrance::Spawn(..) = target { play.reset(Entrance::default()) };
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
                        Update::Turn(side) => animate_with(&animation, 0, || match side {Side::Left => Box::new((0..5).rev().map(|i| repeat(i).take(2)).flatten()), Side::Right => Box::new((0..5).map(|i| repeat(i).take(2)).flatten())}),
                        Update::Fade(inout) => animate_with(&animation, 0, || if inout {Box::new(FADE_IN.iter().cloned())} else {Box::new(FADE_OUT.iter().cloned())}),
                        Update::Burn => animate_with(&animation, 0, || Box::new(atlas::BURN.cycle()) ),
                        Update::Start(Environment::Grease, Some(bottle)) => animate_with(&animation, bottle.get(), 
                            || Box::new((atlas::TIPPING..=atlas::TIPPING).map(|i| repeat(i).take(2)).flatten())
                        ),
                        _ => ()
                    }
                }
                display.show(&sdl2::pixels::Color::RGB(0, 0, 0));
                if !play.dark() {display.show(&backdrop);}
                display.show(&(&play, &animation));
            },
            Outcome::Dead => {
            	animation.borrow_mut().remove(&0);
            	play.reset(match target {Entrance::Flying(side, ..) => Entrance::Spawn(side), target => target})
            }
            Outcome::Leave{destination: Some((to_room, at)), ..} if to_room == room.0 => play.reset(at),
            Outcome::Leave{score, destination} => return Ok((score, destination)),
            _ => ()
        }
        last = Instant::now();
    }
    Err(())
}

use std::collections::HashMap;

pub fn play(context: &mut crate::App, pics: &HashMap<usize, Texture>, house: &[Room]) -> Result<(u32, NonZero<u16>), ()> {
    let mut score = 0u32;
    let mut room_index = crate::test::START;
    let mut arrive = Entrance::default();
    while let (points, Some((next, at))) = {
    	let room = &house[room_index.get() as usize - 1];
        eprintln!("Object count: {}, room theme: {}", room.objects.len(), room.theme_index());
        run(context, &pics[&(room.theme_index() as usize)], (room_index, room), arrive)?
    } {
        score += points;
        if next.get() as usize > house.len() { eprintln!("Left house for pending room {}", next.get()); return Err(()) }
        (room_index, arrive) = (next, at);
    } 
    eprintln!("Left house to {room_index:?}");
    Ok((score, room_index))
}
