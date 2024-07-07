use sdl2::{keyboard::{KeyboardState, Scancode}, render::Texture};
use glider::{Entrance, Input, Outcome, Room, Side, Update};
use crate::{atlas, draw::{Animations, Frame, Scribe, Visible}, room::{SCREEN_HEIGHT, SCREEN_WIDTH}};
use std::{time::{Duration, Instant}, num::NonZero};

const FADE_IN: &[usize] = &[3, 4, 3, 4, 5, 4, 5, 6, 5, 6, 7, 6, 7, 8, 7, 8, 9];
const FADE_OUT: &[usize] = &[9, 8, 9, 8, 7, 8, 7, 6, 7, 6, 5, 6, 5, 4, 5, 4, 3];

fn animate_with<F: FnOnce() -> Frame>(list: &Animations, id: u8, loader: F) {
	let mut list = list.borrow_mut();
    if !list.contains_key(&id) {list.insert(id, loader());}
}

pub fn run(context: &mut crate::App, theme: &Texture, room: (NonZero<u16>, &Room), target: Entrance) -> Result<(u32, Option<(NonZero<u16>, Entrance)>), ()> {
    let display = &mut context.display;
    let loader = display.texture_creator();

    let mut backdrop = loader.create_texture_target(None, SCREEN_WIDTH, SCREEN_HEIGHT).expect("Failed to create backdrop texture");
    let _ = display.with_texture_canvas(&mut backdrop,
        |display| {
            let mut display = (display, &context.sprites);
            display.show(&(theme, room.1.tile_order));
            for object in room.1.objects.iter().filter(|&object| !object.dynamic()) {
                display.show(object);
            }
        }
    );
    let mut display = (display, &context.sprites);
    let mut play = room.1.start(target);
    if let Entrance::Spawn(..) = target { play.reset(Entrance::default()) };
    let animation = Animations::default();

    let mut last = Instant::now();
    'game: loop {
        while last.elapsed() < Duration::from_millis(33) {}
        let mut inputs = Vec::new();
        for event in context.events.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit{..} => break 'game,
                _ => ()
            }
        }
        let keys = KeyboardState::new(&context.events);
        if keys.is_scancode_pressed(Scancode::Right) {inputs.push(Input::Go(Side::Right))};
        if keys.is_scancode_pressed(Scancode::Left) {inputs.push(Input::Go(Side::Left))};
        let result = play.frame(&inputs);

        match result {
            Outcome::Continue(updates) => {
                for update in updates.into_iter().flatten() {
                    match update {
                        Update::Fade(inout) => animate_with(&animation, 0, || if inout {Box::new(FADE_IN.iter().cloned())} else {Box::new(FADE_OUT.iter().cloned())}),
                        Update::Burn => animate_with(&animation, 0, || Box::new(atlas::BURN.cycle()) ),
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
