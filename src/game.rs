use sdl2::{keyboard::{KeyboardState, Scancode}, render::Texture};
use glider::{Entrance, Input, Outcome, Room, Side, Update};
use crate::{atlas, draw::{Animations, Frame, Scribe}, room::{SCREEN_HEIGHT, SCREEN_WIDTH}};
use std::time::{Duration, Instant};

const FADE_IN: &[usize] = &[3, 4, 3, 4, 5, 4, 5, 6, 5, 6, 7, 6, 7, 8, 7, 8, 9];
const FADE_OUT: &[usize] = &[9, 8, 9, 8, 7, 8, 7, 6, 7, 6, 5, 6, 5, 4, 5, 4, 3];

fn animate_with<F: FnOnce() -> Frame>(list: &mut Animations, id: u8, loader: F) {
	if !list.contains_key(&id) {list.insert(id, loader());}
}

pub fn run(context: &mut crate::App, theme: &Texture, room: (usize, &Room), target: Entrance) -> Result<(u32, Option<(i16, Entrance)>), ()> {
    let display = &mut context.display;
    let loader = display.texture_creator();

    let mut backdrop = loader.create_texture_target(None, SCREEN_WIDTH, SCREEN_HEIGHT).expect("Failed to create backdrop texture");
    let _ = display.with_texture_canvas(&mut backdrop,
        |display| {
            display.draw_wall(&theme, &room.1.tile_order);
            for object in room.1.objects.iter().filter(|&object| !object.dynamic()) {
                display.draw_object(object, &context.sprites);
            }
        }
    );
    let mut play = room.1.start(target);
    if let Entrance::Spawn(..) = target { play.reset(Entrance::default()) };
    let mut animation = HashMap::<u8, Box<dyn Iterator<Item = usize>>>::new();

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
                        Update::Fade(inout) => animate_with(&mut animation, 0, || if inout {Box::new(FADE_IN.iter().cloned())} else {Box::new(FADE_OUT.iter().cloned())}),
                        Update::Burn => animate_with(&mut animation, 0, || Box::new(atlas::BURN.cycle()) ),
                        _ => ()
                    }
                }
                display.draw_room(&play, &mut animation, &context.sprites, &backdrop)
            },
            Outcome::Dead => {
            	animation.remove(&0);
            	play.reset(match target {Entrance::Flying(side, ..) => Entrance::Spawn(side), target => target})
            }
            Outcome::Leave{destination: Some((to_room, at)), ..} if to_room as usize == room.0 => {eprintln!("jumping to {to_room}, {at:?}"); play.reset(at)},
            Outcome::Leave{score, destination} => return Ok((score, destination)),
            _ => ()
        }
        last = Instant::now();
    }
    Err(())
}

use std::collections::HashMap;

pub fn play(context: &mut crate::App, pics: &HashMap<usize, Texture>, house: &[Room]) -> Result<(u32, usize), ()> {
    let mut score = 0u32;
    let mut room = 0usize;
    let mut arrive = Entrance::default();
    while let (points, Some((next, at))) = {
        eprintln!("Object count: {}", house[room as usize].objects.len());
        run(context, &pics[&(house[room as usize].theme_index() as usize)], (room + 7, &house[room]), arrive)?
    } {
        score += points;
        (room, arrive) = match at {
        	Entrance::Air => (next as usize, at),
            Entrance::Flying(..) => {
                let Some(index) = room.checked_add_signed(next as isize) else { return Err(()) };
                if index >= house.len() { return Err(()) }
                (index, at)
            }
            Entrance::Spawn(..) => (next as usize, at)
        };
    }
    Ok((score, room))
}
