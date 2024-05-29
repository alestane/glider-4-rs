use sdl2::{pixels::Color, render::Texture, rect::Rect};
use glider::{Room, ObjectKind};
use crate::{atlas, draw, SCREEN_HEIGHT, SCREEN_WIDTH};

fn appearance(kind: &ObjectKind) -> Option<(&'static str, usize)> {
    Some(match kind {
        ObjectKind::FloorVent { .. } => ("blowers", atlas::UP),
        _ => return None
    })
}

pub fn run(context: &mut crate::App, theme: Texture, room: &Room) {
    let display = &mut context.display;
    let loader = display.texture_creator();

    let mut backdrop = loader.create_texture_target(None, SCREEN_WIDTH, SCREEN_HEIGHT).expect("Failed to create backdrop texture");
    let _ = display.with_texture_canvas(&mut backdrop, 
        |display| {
            draw::wall(display, &theme, &room.tile_order);
            for object in room.objects.iter().filter(|&object| !object.dynamic()) {
                match appearance(&object.object_is) {
                    Some((name, index)) => {
                        let (wedge, tex) = context.sprites.get(name);
                        let _ = display.copy(tex, Some(wedge[index].into()), Some(crate::space::Rect::from(object.bounds).into()));
                    }
                    None => {
                        draw::thing(display, object, &context.sprites);
                    }
                }
            }
        }
    );
    'game: loop {
        for event in context.events.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit{..} => break 'game,
                _ => ()
            }
        }
        display.set_draw_color(Color::RGB(0, 0, 0));
        display.clear();
        let _ = display.copy(&backdrop, None, None);
        display.present();
    }
}