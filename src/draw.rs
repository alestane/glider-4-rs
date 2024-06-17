use std::{collections::HashMap, num::NonZero};

use glider::{Side, Object, ObjectKind, prelude::Enemy};
use sdl2::{pixels::{Color, PixelFormatEnum}, rect::{Point, Rect}, render::{BlendMode, Canvas, RenderTarget, Texture}, surface::Surface, video::Window};
use crate::{room::{SCREEN_HEIGHT, SCREEN_WIDTH, VERT_FLOOR}, space, atlas::{self, Atlas}, resources};

pub type Frame = Box<dyn Iterator<Item = usize>>;
pub type Animations = HashMap<u8, Frame>;

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
        ObjectKind::Battery(..) => ("collectible", atlas::BATTERY),
        ObjectKind::FloorVent { .. } => ("blowers", atlas::UP),
        ObjectKind::CeilingDuct { .. } => ("blowers", atlas::DUCT),
        ObjectKind::Candle { .. } => ("blowers", atlas::CANDLE),
        ObjectKind::Macintosh => ("visual", atlas::COMPUTER),
        ObjectKind::Books => ("visual", atlas::BOOKS),
        ObjectKind::Painting => ("visual", atlas::PAINTING),
        ObjectKind::Paper(..) => ("collectible", atlas::PAPER),
        _what => return None
    })
}

fn descreen(target: &mut Surface, width: u32) {
    target.without_lock_mut().map(|pixels| {
        for (v, row) in pixels.as_chunks_mut::<4>().0.chunks_mut(width as usize).enumerate() {
            for pair in row[(v % 2)..].chunks_mut(2) {
                pair[0] = [0x00; 4];
            }
        }
    });
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

    descreen(&mut shadow, width);
    Ok(shadow)
}

fn shelf_shadow(width: NonZero<u32>, height: NonZero<u32>) -> Result<Surface<'static>, Box<dyn std::error::Error>> {
    let (width, height) = (width.get(), height.get());
    let mut shadow = {
        let mut canvas = Surface::new(width + 15, height + 15, PixelFormatEnum::RGBA8888)?.into_canvas()?;
        canvas.set_draw_color((0, 0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(BLACK);
        for left in 0..15 {
            canvas.fill_rect(Rect::new(left, 15 - left, width, height))?;
        }
        canvas.into_surface()
    };
    descreen(&mut shadow, width + 15);
    Ok(shadow)
}

fn cabinet_shadow(height: NonZero<u32>) -> Result<Surface<'static>, Box<dyn std::error::Error>> {
    let height = height.get();
    let mut shadow = {
        let mut canvas = Surface::new(15, height, PixelFormatEnum::RGBA8888)?.into_canvas()?;
        canvas.set_draw_color((0, 0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(BLACK);
        for h in 0..15 {
            canvas.draw_line((h, 20 - h), (h, (height as i32 - 10) + (h * 10) / 15))?;
        }
        canvas.into_surface()
    };
    descreen(&mut shadow, 15);
    Ok(shadow)
}

fn window_shadow(width: NonZero<u32>, height: NonZero<u32>) -> Result<Surface<'static>, Box<dyn std::error::Error>> {
    let (width, height) = (width.get(), height.get());
    let mut shadow = {
        let mut canvas = Surface::new(width + 10, height + 5, PixelFormatEnum::RGBA8888)?.into_canvas()?;
        canvas.set_draw_color((0, 0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(BLACK);
        for left in 0..5 {
            canvas.fill_rect(Rect::new(left + 5, 5 - left, width, height))?;
            canvas.draw_line((left, 10 - left), (left, 15 + left))?;
            canvas.draw_line((left, height as i32 - (5 + left)), (left, height as i32 + left))?;
        }
        canvas.into_surface()
    };
    descreen(&mut shadow, width + 10);
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
    fn outline_rect(&mut self, bounds: Rect, fill: impl Into<Color>) -> Result<(), String>;
    fn limn_rect(&mut self, bounds: Rect, fill: impl Into<Color>, hilite: impl Into<Color>) -> Result<(), String>;
    fn sink_rect(&mut self, bounds: Rect, fill: impl Into<Option<Color>>) -> Result<(), String>;
    fn draw_wall(&mut self, theme: &Texture, tiles: &[u16]);
    fn draw_table(&mut self, bounds: space::Rect, sprite: (space::Rect, &Texture)) -> Result<(), String>;
    fn draw_shelf(&mut self, bounds: space::Rect, sprite: (space::Rect, &Texture)) -> Result<(), String>;
    fn draw_cabinet(&mut self, bounds: space::Rect) -> Result<(), String>;
    fn draw_mirror(&mut self, bounds: space::Rect) -> Result<(), String>;
    fn draw_window(&mut self, bounds: space::Rect, is_open: bool) -> Result<(), String>;
    fn draw_thing(&mut self, object: &Object, sprites: &Atlas);
    fn draw_object(&mut self, object: &Object, atlas: &atlas::Atlas);
    fn draw_sprite(&mut self, bounds: space::Rect, name: &str, index: usize, sprites: &Atlas);
    fn draw_room(&mut self, play: &glider::Play, times: &mut HashMap<u8, Box<dyn Iterator<Item = usize>>>, sprites: &Atlas, backdrop: &Texture);
}

impl<R:RenderTarget, T> Scribe for Canvas<R> where Self: Illuminator<Builder = sdl2::render::TextureCreator<T>> {
    fn draw(&mut self, pixels: &Texture, source: impl Into<Option<Rect>>, dest: impl Into<Option<Rect>>) {
        self.copy(pixels, source, dest)
            .expect("failed to draw to canvas");
    }

    fn outline_rect(&mut self, bounds: Rect, fill: impl Into<Color>) -> Result<(), String> {
        self.set_draw_color(fill.into());
        self.fill_rect(bounds)?;
        self.set_draw_color(BLACK);
        self.draw_rect(bounds)
    }

    fn limn_rect(&mut self, bounds: Rect, fill: impl Into<Color>, hilite: impl Into<Color>) -> Result<(), String> {
        self.outline_rect(bounds, fill)?;
        self.set_draw_color(hilite);
        self.draw_lines([
            Point::new(bounds.left() + 1, bounds.top() +1),
            Point::new(bounds.right() - 2, bounds.top() + 1),
            Point::new(bounds.right() - 2, bounds.bottom() - 2),
        ].as_ref())
    }

    fn sink_rect(&mut self, bounds: Rect, fill: impl Into<Option<Color>>) -> Result<(), String> {
        if let Some(fill) = fill.into() {
            self.outline_rect(bounds, fill)?;
        }
        self.set_draw_color(BROWN_LT);
        self.draw_lines([
            Point::new(bounds.left() - 1, bounds.top()),
            Point::new(bounds.left() - 1, bounds.bottom()),
            Point::new(bounds.right() - 1, bounds.bottom()),
        ].as_ref())?;
        self.set_draw_color(GRAY_DK);
        self.draw_lines([
            Point::new(bounds.left(), bounds.top() - 1),
            Point::new(bounds.right(), bounds.top() - 1),
            Point::new(bounds.right(), bounds.bottom()),
        ].as_ref())
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

    fn draw_shelf(&mut self, bounds: space::Rect, sprite: (space::Rect, &Texture)) -> Result<(), String> {
        let bounds: Rect = bounds.into();

        let shadow = unsafe { shelf_shadow(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height()))}.or_else(|e| Err(e.to_string()))?;
        let builder = self.get_builder();
        self.copy(&shadow.as_texture(&builder).map_err(|e| e.to_string())?, None, Rect::new(bounds.left() - 15, bounds.top(), bounds.width() + 15, bounds.height() + 15))?;

        self.set_draw_color(BROWN_LT);
        self.fill_rect(bounds)?;
        self.draw_line((bounds.left() + 1, bounds.bottom() - 2), (bounds.right() - 1, bounds.bottom() -2))?;
        self.set_draw_color(WHITE);
        self.draw_line((bounds.left() + 1, bounds.top() + 1), (bounds.right() - 2, bounds.top() + 1))?;

        let source: Rect = sprite.0.into();
        self.copy(sprite.1, source, Rect::new(bounds.left() + 15, bounds.bottom() - 2, source.width(), source.height()))?;
        self.copy(sprite.1, source, Rect::new(bounds.right() - (15 + source.width() as i32), bounds.bottom() - 2, source.width(), source.height()))?;

        Ok(())
    }

    fn draw_cabinet(&mut self, bounds: space::Rect) -> Result<(), String> {
        let bounds: Rect = bounds.into();
        let off = if bounds.bottom() > 280 {
            {
                let bounds = Rect::new(bounds.left(), bounds.top(), bounds.width(), bounds.height() - 5);
                self.set_draw_color(BROWN);
                self.set_blend_mode(BlendMode::Blend);
                self.fill_rect(bounds)?;
                self.set_draw_color(BLACK);
                self.draw_rect(bounds)?;
            }
            {
                let bounds = Rect::new(bounds.left() + 2, bounds.bottom() - 5, bounds.width() - 5, 5);
                self.fill_rect(bounds)?;
                self.draw_rect(bounds)?;
            }
            {
                let bounds = Rect::new(bounds.left() - 2, bounds.top(), bounds.width() + 4, 7);
                self.set_draw_color(BROWN_LT);
                self.fill_rect(bounds)?;
                self.set_draw_color(BLACK);
                self.draw_rect(bounds)?;
                self.draw_line((bounds.left() + 2, bounds.bottom()), (bounds.right() - 3, bounds.bottom()))?;
            }
            {
                let shadow = cabinet_shadow(unsafe{NonZero::new_unchecked(bounds.height())}).map_err(|e| e.to_string())?;
                let builder = self.get_builder();
                self.copy(&shadow.as_texture(&builder).map_err(|e| e.to_string())?, None, Rect::new(bounds.left() - 15, bounds.top(), 15, bounds.height()))?;
            }
            5u32
        } else {
            {
                let shadow = unsafe { shelf_shadow(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height())) }.map_err(|e| e.to_string())?;
                let builder = self.get_builder();
                self.copy(&shadow.as_texture(&builder).map_err(|e| e.to_string())?, None, Rect::new(bounds.left() - 15, bounds.top(), bounds.width() + 15, bounds.height() + 15))?;
            }
            {
                self.set_draw_color(BROWN);
                self.fill_rect(bounds)?;
                self.set_draw_color(BLACK);
                self.draw_rect(bounds)?;
            }
            0
        };
        let panels = bounds.width().max(48) / 48;
        let width = (bounds.width() - (panels + 1) * 5) / panels;
        for h in 0..panels {
            let bounds = Rect::new(bounds.left() + 8 + h as i32 * (width as i32 + 5), bounds.top() + 8i32.saturating_add_unsigned(off), width - 7, bounds.height() - (17 + off * 2));
            self.set_draw_color(BROWN_LT);
            self.draw_lines([Point::new(bounds.left(), bounds.top()), Point::new(bounds.left(), bounds.bottom()), Point::new(bounds.right(), bounds.bottom())].as_ref())?;
            self.set_draw_color(BLACK);
            self.draw_lines([Point::new(bounds.right(), bounds.bottom()), Point::new(bounds.right(), bounds.top()), Point::new(bounds.left(), bounds.top())].as_ref())?;
        }
        Ok(())
    }

    fn draw_mirror(&mut self, bounds: space::Rect) -> Result<(), String> {
        let outer: Rect = bounds.into();
        let inner = Rect::new(outer.left() + 3, outer.top() + 3, outer.width() - 6, outer.height() - 6);
        self.set_draw_color(BROWN);
        self.fill_rect(outer)?;
        self.set_draw_color(WHITE);
        self.fill_rect(inner)?;
        self.set_draw_color(BLACK);
        self.draw_rect(outer)?;
        self.draw_rect(inner)?;
        Ok(())
    }

    fn draw_window(&mut self, bounds: space::Rect, is_open: bool) -> Result<(), String> {
        let bounds: Rect = bounds.into();
        {
            let shadow = unsafe { window_shadow(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height())) }
                .map_err(|e| e.to_string())?;
            let builder = self.get_builder();
            self.draw(&shadow.as_texture(&builder).map_err(|e| e.to_string())?, None, Rect::new(bounds.left() - 10, bounds.top(), bounds.width() + 10, bounds.height() + 5));
        }
        self.limn_rect(bounds, BROWN, BROWN_LT)?;
        self.limn_rect(Rect::new(bounds.left() - 4, bounds.top(), bounds.width() + 8, 6), BROWN, BROWN_LT)?;
        self.limn_rect(Rect::new(bounds.left() - 2, bounds.top() + 6, bounds.width() + 4, 4), BROWN, BROWN_LT)?;
        self.limn_rect(Rect::new(bounds.left() - 4, bounds.bottom() - 6, bounds.width() + 8, 6), BROWN, BROWN_LT)?;
        self.limn_rect(Rect::new(bounds.left() - 2, bounds.bottom() - 10, bounds.width() + 4, 4), BROWN, BROWN_LT)?;
        let casing = Rect::new(bounds.left() + 8, bounds.top() + 16, bounds.width() - 16, bounds.height() - 32);
        self.sink_rect(casing, BROWN)?;
        {
            let pane = Rect::new(casing.left(), casing.top(), casing.width(), casing.height() / 2 + 2);

            self.outline_rect(pane, BROWN)?;
            let pane = Rect::new(pane.left() + 6, pane.top() + 6, pane.width() - 12, pane.height() - 12);
            self.sink_rect(pane, None)?;
            let pane = Rect::new(pane.left() + 2, pane.top() + 2, pane.width() - 4, pane.height() - 4);
            self.sink_rect(pane, None)?;
            let pane = Rect::new(pane.left() + 2, pane.top() + 2, pane.width() - 4, pane.height() - 4);
            self.sink_rect(pane, BLACK)?;
        }
        let height = casing.height() / 2 + 2;
        {
            let pane = Rect::new(casing.left(), casing.top() + height as i32, casing.width(), casing.height() - height);
            self.set_draw_color(BLACK);
            self.fill_rect(pane)?;
            let pane = Rect::new(casing.left(), casing.top() + if is_open {26} else {height as i32 - 4}, casing.width(), height);
            self.outline_rect(pane, BROWN)?;
            let pane = Rect::new(pane.left() + 6, pane.top() + 6, pane.width() - 12, pane.height() - 12);
            self.sink_rect(pane, None)?;
            let pane = Rect::new(pane.left() + 2, pane.top() + 2, pane.width() - 4, pane.height() - 4);
            self.sink_rect(pane, None)?;
            let pane = Rect::new(pane.left() + 2, pane.top() + 2, pane.width() - 4, pane.height() - 4);
            self.sink_rect(pane, BLACK)?;
        }
        Ok(())
    }

    fn draw_thing(&mut self, object: &Object, decor: &Atlas) {
        match
            match object.object_is {
                ObjectKind::Table => {
                    let (frame, pixels) = decor.get("visual");
                    self.draw_table(object.bounds.into(), (frame[atlas::TABLE], pixels))
                }
                ObjectKind::Shelf => {
                    let (frame, pixels) = decor.get("visual");
                    self.draw_shelf(object.bounds.into(), (frame[atlas::SHELF], pixels))
                }
                ObjectKind::Cabinet => {
//                    let (frame, pixels) = decor.get("visual");
                    self.draw_cabinet(object.bounds.into())
                }
                ObjectKind::Mirror => {
                    self.draw_mirror(object.bounds.into())
                }
                ObjectKind::Window => {
                    self.draw_window(object.bounds.into(), object.is_on)
                }
                ObjectKind::Bonus(..) => Ok(()),
                what => Err(format!("Unimplemented object: {what:?}")),
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

    fn draw_room(&mut self, play: &glider::Play, times: &mut Animations, sprites: &Atlas, backdrop: &Texture) {
        fn advance(lookup: &mut Animations, id: u8) -> Option<usize> {
            let index = lookup.get_mut(&id).and_then(|a| a.next());
            if index.is_none() {
                lookup.remove(&id);
            }
            index
        }

        self.set_draw_color(Color::RGB(0, 0, 0));
        self.clear();
        self.draw(&backdrop, None, None);
        let (mut player_position, facing, backward) = play.player();
        let (slides, pixels) = sprites.get(match facing {Side::Left => "glider.left", Side::Right => "glider.right"} );
        let frame: sdl2::rect::Rect = slides[advance(times, 0).unwrap_or(if backward {atlas::TIPPED} else {atlas::LEVEL})].into();
        if frame.height() > 20 {player_position.1 -= frame.height() as i16 / 2 - 10};
        for item in play.active_items().filter(|&o| o.object_is == ObjectKind::Mirror) {
            let bounds: Rect = space::Rect::from(item.bounds).into();
            self.set_clip_rect(Rect::new(bounds.left() + 3, bounds.top() + 3, bounds.width() - 6, bounds.height() - 6));
            self.draw(pixels, frame, frame.centered_on((player_position.0 as i32 - 16, player_position.1 as i32  - 32)));
        }
        self.set_clip_rect(None);
        for item in play.active_items().filter(|&o| o.dynamic()) {
            self.draw_object(item, sprites);
        }
        for (id, hazard, bounds) in play.active_hazards() {
            match hazard {
                Enemy::Flame => {
                    let frame = if let Some(frame) = times.get_mut(&id).and_then(|seq| seq.next()) {
                        frame
                    } else {
                        let skip = id as usize % 3;
                        let mut c = (0..3).cycle();
                        c.advance_by(skip).ok();
                        let frame = unsafe{ c.next().unwrap_unchecked() };
                        times.insert(id, Box::new(c));
                        frame
                    };
                    self.draw_sprite(space::Rect::from(bounds).into(), "fire", frame, sprites);
                }
                _ => ()
            }
        }
        self.draw(pixels, frame, frame.centered_on((player_position.0 as i32, player_position.1 as i32)));
        let frame: sdl2::rect::Rect = slides[atlas::SHADOW].into();
        self.draw(pixels, frame, frame.centered_on((player_position.0 as i32, (VERT_FLOOR + frame.height() / 2) as i32)));
        self.present();
    }
}
