use sdl2::{render::Texture, keyboard::{KeyboardState, Scancode}};
use glider::{Entrance, Input, Room, Side};
use crate::{draw::Scribe, SCREEN_HEIGHT, SCREEN_WIDTH};

pub fn run(context: &mut crate::App, theme: Texture, room: &Room) {
    let display = &mut context.display;
    let loader = display.texture_creator();

    let mut backdrop = loader.create_texture_target(None, SCREEN_WIDTH, SCREEN_HEIGHT).expect("Failed to create backdrop texture");
    let _ = display.with_texture_canvas(&mut backdrop, 
        |display| {
            display.draw_wall(&theme, &room.tile_order);
            for object in room.objects.iter().filter(|&object| !object.dynamic()) {
                display.draw_object(object, &context.sprites);
            }
        }
    );
    let mut play = room.start(Entrance::Flying(Side::Left), true, true);
    
    'game: loop {
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
        play.frame(&inputs);
        display.draw_room(&play, &context.sprites, &backdrop);
    }
}