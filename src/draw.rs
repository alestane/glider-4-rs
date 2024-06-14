use std::num::NonZero;

use glider::{Side, Object, ObjectKind};
use sdl2::{pixels::{Color, PixelFormatEnum}, rect::Rect, render::{BlendMode, Canvas, RenderTarget, Texture}, surface::Surface, video::Window};
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH, VERT_FLOOR, space, atlas::{self, Atlas}, resources};

const BLACK     : Color = Color::RGB(0x00, 0x00, 0x00);
const WHITE     : Color = Color::RGB(0xFF, 0xFF, 0xFF);
const YELLOW    : Color = Color::RGB(0xFF, 0xFF, 0x00);
const VIOLET    : Color = Color::RGB(0xFF, 0x00, 0xFF);
const RED       : Color = Color::RGB(0xFF, 0x00, 0x00);
const BLUE_LT   : Color = Color::RGB(0x00, 0xFF, 0xFF);
const BROWN     : Color = Color::RGB(0x56, 0x2D, 0x05);
const BROWN_LT  : Color = Color::RGB(0x91, 0x71, 0x3A);
const GRAY_DK   : Color = Color::RGB(0x80, 0x80, 0x80);
const BLUE      : Color = Color::RGB(0x00, 0x00, 0xFF);
const GREEN_LT  : Color = Color::RGB(0x1F, 0xB8, 0x14);

fn appearance(kind: &ObjectKind) -> Option<(&'static str, usize)> {
    Some(match kind {
        ObjectKind::Clock(_) => ("collectible", atlas::CLOCK),
        ObjectKind::FloorVent { .. } => ("blowers", atlas::UP),
        ObjectKind::Macintosh => ("visual", atlas::COMPUTER),
        what => { eprintln!("no sprite: {what:?}"); return None }
    })
}

fn table_shadow(width: NonZero<u32>, height: NonZero<u32>) -> Result<Surface<'static>, Box<dyn std::error::Error>> {
    let (width, height) = (width.get(), height.get());
    let mut shadow = {
        let mut canvas = Surface::new(width, height, PixelFormatEnum::RGBA8888)?.into_canvas()?;
        let builder = canvas.texture_creator();
        let mut circle = builder.create_texture_static(PixelFormatEnum::ABGR8888, 256, 256)?;
        circle.update(None, resources::CIRCLE, 256 * 4)?;
        circle.set_blend_mode(BlendMode::Blend);
        canvas.copy(&circle, None, None)?;
        canvas.into_surface()
    };

    shadow.without_lock_mut().map(|pixels| {
        for (v, row) in pixels.as_chunks_mut::<4>().0.chunks_mut(width as usize).enumerate() {
            for pair in row[(v % 2)..].chunks_mut(2) {
                    pair[0] = [0x00; 4];
            }
        }
    }); 
    Ok(shadow)
}

trait Illuminator {
    type Builder;
    fn get_builder(&self) -> Self::Builder;
}

impl Illuminator for Canvas<Window> {
    type Builder = sdl2::render::TextureCreator<sdl2::video::WindowContext>;
    fn get_builder(&self) -> Self::Builder {
        self.texture_creator()
    }
}

pub trait Scribe {
    fn draw(&mut self, pixels: &Texture, source: impl Into<Option<Rect>>, dest: impl Into<Option<Rect>>);
    fn draw_wall(&mut self, theme: &Texture, tiles: &[u16]);
    fn draw_table(&mut self, bounds: space::Rect, sprite: (space::Rect, &Texture)) -> Result<(), String>;
    fn draw_thing(&mut self, object: &Object, sprites: &Atlas);
    fn draw_object(&mut self, object: &Object, atlas: &atlas::Atlas);
    fn draw_sprite(&mut self, bounds: space::Rect, name: &str, index: usize, sprites: &Atlas);
    fn draw_room(&mut self, play: &glider::Play, sprites: &Atlas, backdrop: &Texture);
}    

impl<R:RenderTarget, T> Scribe for Canvas<R> where Self: Illuminator<Builder = sdl2::render::TextureCreator<T>> {
    fn draw(&mut self, pixels: &Texture, source: impl Into<Option<Rect>>, dest: impl Into<Option<Rect>>) {
        self.copy(pixels, source, dest)
            .expect("failed to draw to canvas");
    }    

    fn draw_wall(&mut self, theme: &Texture, tiles: &[u16]) {
        const TILE_WIDTH: u32 = SCREEN_WIDTH / 8;
        for (index, &slice) in tiles.iter().enumerate() {
            self.draw(&theme, 
                Rect::new(slice as i32 * TILE_WIDTH as i32, 0, TILE_WIDTH, SCREEN_HEIGHT), 
                Rect::new(index as i32 * TILE_WIDTH as i32, 0, TILE_WIDTH, SCREEN_HEIGHT)
            );    
        }    
    }    

    fn draw_table(&mut self, bounds: space::Rect, sprite: (space::Rect, &Texture)) -> Result<(), String> {
        let bounds = sdl2::rect::Rect::from(bounds);
        self.set_draw_color(BROWN);
        self.fill_rect(bounds)?;
        self.set_draw_color(BLACK);
        self.draw_rect(bounds)?;
        self.draw_line((bounds.left() + 1, bounds.bottom() - 2), (bounds.right() - 1, bounds.bottom() - 2))?;
        self.set_draw_color(BROWN_LT);
        self.draw_line((bounds.left() + 1, bounds.top() + 1), (bounds.right() - 2, bounds.top() + 1))?;
        let builder = self.get_builder();
        //let shadow = unsafe { table_shadow(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height())) }
        //    .map_err(|e| e.to_string())?;
        let drop = -(bounds.top().saturating_sub_unsigned(VERT_FLOOR));
        let shadow_bounds = { let mut r = bounds; r.offset(drop / -5, drop); r};
        let shadow = unsafe { table_shadow(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height()))}.or_else(|e| Err(e.to_string()))?;
        self.copy(&shadow.as_texture(&builder).map_err(|e| e.to_string())?, None, shadow_bounds)?;
        
        let center = bounds.center().x();
        let post = Rect::new(center - 2, bounds.bottom(), 5, (VERT_FLOOR - 2).saturating_add_signed(-bounds.bottom()));
        self.set_draw_color(BLACK);
        self.fill_rect(post)?;
        self.set_draw_color(WHITE);
        self.draw_line((center + 1, post.top()), (center + 1, post.bottom()))?;
        self.set_draw_color(BROWN_LT);
        self.draw_line((center, post.top()), (center, post.bottom()))?;
        
        let frame: Rect = sprite.0.into();
        self.copy(sprite.1, frame, frame.centered_on((center, VERT_FLOOR as i32 + 4)))?;
        
        Ok(())
    }

    fn draw_thing(&mut self, object: &Object, decor: &Atlas) {
        match 
            match object.object_is {
                ObjectKind::Table => {
                    let (frame, pixels) = decor.get("visual");
                    self.draw_table(object.bounds.into(), (frame[atlas::TABLE], pixels))
                }
                _ => Ok(())
            } {
                Err(e) => eprintln!("{e}"),
                _ => ()
        }
    }

    fn draw_sprite(&mut self, bounds: space::Rect, name: &str, index: usize, sprites: &Atlas) {
        let (wedge, tex) = sprites.get(name);
        self.draw(tex, wedge[index], bounds);
    }

    fn draw_object(&mut self, object: &Object, atlas: &Atlas) {
        match appearance(&object.object_is) {
            Some((name, index)) => self.draw_sprite(object.bounds.into(), name, index, atlas),
            None => self.draw_thing(object, atlas),
        }
    }

    fn draw_room(&mut self, play: &glider::Play, sprites: &Atlas, backdrop: &Texture) {
        self.set_draw_color(Color::RGB(0, 0, 0));
        self.clear();
        self.draw(&backdrop, None, None);
        for item in play.active_items().filter(|&o| o.dynamic()) {
            self.draw_object(item, sprites);
        }
        let (position, facing, backward) = play.player();
        let (slides, pixels) = sprites.get(match facing {Side::Left => "glider.left", Side::Right => "glider.right"} );
        let frame: sdl2::rect::Rect = slides[if backward {atlas::TIPPED} else {atlas::LEVEL}].into();
        self.draw(pixels, frame, frame.centered_on((position.0 as i32, position.1 as i32)));
        let frame: sdl2::rect::Rect = slides[atlas::SHADOW].into();
        self.draw(pixels, frame, frame.centered_on((position.0 as i32, (crate::VERT_FLOOR + frame.height() / 2) as i32)));
        self.present();
    }
}    
