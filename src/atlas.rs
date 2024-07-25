use std::{ops::{Index, Range}, collections::HashMap};

use sdl2::{image::LoadTexture, pixels::PixelFormatEnum, surface::Surface};

use crate::{room, space::Rect};

const ITEMS: [Rect; 100] = [
    Rect::new_signed(256, 0, 304, 11),      // shadow right

    Rect::new_signed(0,  0, 48, 20),        // forward right
    Rect::new_signed(0, 21, 48, 41),        // backward right

    Rect::new_signed(414,  97, 462, 117),   // fade right
    Rect::new_signed(414, 118, 462, 138),
    Rect::new_signed(414, 139, 462, 159),
    Rect::new_signed(414, 160, 462, 180),
    Rect::new_signed(414, 181, 462, 201),
    Rect::new_signed(414, 202, 462, 222),
    Rect::new_signed(414, 223, 462, 243),

    Rect::new_signed(256,  24, 304, 60),    // burn right
    Rect::new_signed(256,  61, 304, 97),

    Rect::new_signed(256, 12, 304, 23),     // shadow left

    Rect::new_signed(0, 42, 48, 62),        // forward left
    Rect::new_signed(0, 63, 48, 83),        // backward left

    Rect::new_signed(463,  97, 511, 117),   // fade left
    Rect::new_signed(463, 118, 511, 138),
    Rect::new_signed(463, 139, 511, 159),
    Rect::new_signed(463, 160, 511, 180),
    Rect::new_signed(463, 181, 511, 201),
    Rect::new_signed(463, 202, 511, 222),
    Rect::new_signed(463, 223, 511, 243),

    Rect::new_signed(256,  98, 304, 134),   // burn left
    Rect::new_signed(256, 135, 304, 171),

    Rect::new_signed(208,   0, 256, 20),    // glider spin
    Rect::new_signed(208,  21, 256, 41),
    Rect::new_signed(208,  42, 256, 62),
    Rect::new_signed(208,  63, 256, 83),
    Rect::new_signed(208,  84, 256, 104),
    Rect::new_signed(208, 105, 256, 125),

    Rect::new_signed(0, 84, 48, 96),        // ceiling vent
    Rect::new_signed(0, 97, 48, 110),       // ceiling duct
    Rect::new_signed(0, 111, 48, 124),      // floor vent
    Rect::new_signed(0, 175, 41, 205),      // kettle
    Rect::new_signed(0, 206, 35, 261),      // fan left
    Rect::new_signed(0, 262, 35, 316),      // fan right
    Rect::new_signed(112, 30, 144, 51),     // candle

    Rect::new_signed(0, 125, 48, 146),      // paper
    Rect::new_signed(112, 0, 144, 29),      // clock
    Rect::new_signed(112, 52, 144, 75),     // bands
    Rect::new_signed(144, 55, 160, 81),     // battery

    Rect::new_signed(48, 23, 112, 45),      // table base
    Rect::new_signed(49, 46, 112, 70),      // shredder
    Rect::new_signed(48, 71, 112, 126),     // books
    Rect::new_signed(48, 127, 112, 297),    // guitar
    Rect::new_signed(192, 71, 208, 100),    // shelf stanchion
    Rect::new_signed(448, 270, 511, 341),   // wastebasket
    Rect::new_signed(408, 0, 510, 92),      // painting
    Rect::new_signed(256, 209, 301, 267),   // macintosh

    Rect::new_signed(142, 0, 160, 26),      // light switch
    Rect::new_signed(144, 27, 162, 54),     // thermostat
    Rect::new_signed(160, 264, 192, 289),   // outlet
    Rect::new_signed(160, 290, 192, 315),   // spark 1
    Rect::new_signed(160, 316, 192, 341),   // spark 2
    Rect::new_signed(144, 082, 162, 108),   // button switch

    Rect::new_signed(  0, 147,  38, 174),   // toaster
    Rect::new_signed(304,  84, 336, 115),   // toast
    Rect::new_signed(304, 116, 336, 115),   //   "
    Rect::new_signed(304, 148, 336, 179),
    Rect::new_signed(304, 180, 336, 211),
    Rect::new_signed(304, 212, 336, 243),
    Rect::new_signed(304, 244, 336, 275),

    Rect::new_signed(112, 109, 144, 138),   // fishbowl
    Rect::new_signed(144, 109, 160, 125),   // fish
    Rect::new_signed(144, 126, 160, 142),
    Rect::new_signed(144, 143, 160, 159),
    Rect::new_signed(144, 160, 160, 176),

    Rect::new_signed(144, 189, 160, 201),   // candle flame
    Rect::new_signed(144, 202, 160, 214),
    Rect::new_signed(144, 215, 160, 227),

    Rect::new_signed(192, 42, 208, 55),
    Rect::new_signed(192, 0, 208, 13),      // drip
    Rect::new_signed(192, 14, 208, 27),
    Rect::new_signed(192, 28, 208, 41),
    Rect::new_signed(192, 56, 208, 70),

    Rect::new_signed(112, 76, 144, 108),    // basketball

    Rect::new_signed(112, 139, 144, 168),   // grease
    Rect::new_signed(112, 169, 144, 198),
    Rect::new_signed(112, 199, 144, 228),

    Rect::new_signed(304, 0, 368, 22),      // dart
    Rect::new_signed(48, 0, 112, 22),

    Rect::new_signed(304, 276, 336, 308),   // copter
    Rect::new_signed(160, 0, 192, 32),
    Rect::new_signed(160, 33, 192, 65),
    Rect::new_signed(160, 66, 192, 98),
    Rect::new_signed(160, 99, 192, 131),
    Rect::new_signed(160, 132, 192, 164),
    Rect::new_signed(160, 165, 192, 197),
    Rect::new_signed(160, 198, 192, 230),
    Rect::new_signed(160, 231, 192, 263),

    Rect::new_signed(304, 309, 336, 341),
    Rect::new_signed(112, 229, 144, 261),
    Rect::new_signed(112, 262, 144, 294),
    Rect::new_signed(112, 295, 144, 327),

    Rect::new_signed(192, 155, 208, 162),
    Rect::new_signed(192, 163, 208, 170),
    Rect::new_signed(192, 171, 208, 178),

    Rect::new_signed(1, 343, 162, 597),
    Rect::new_signed(163, 343, 324, 597),

    Rect::new_signed(256, 172, 304, 208),
];

const GLIDE_RIGHT   : Range<usize> = 0..12;
const GLIDE_LEFT    : Range<usize> = 12..24;
const GLIDE_TURN    : Range<usize> = 24..30;

pub const SHADOW    : usize = 0;
pub const LEVEL     : usize = 1;
pub const TIPPED    : usize = 2;
pub const FADE      : Range<usize> = 3..10;
pub const BURN      : Range<usize> = 10..12;

const AIR           : Range<usize> = 30..37;
pub const DOWN      : usize = 0;
pub const DUCT      : usize = 1;
pub const UP        : usize = 2;
pub const KETTLE    : usize = 3;
pub const FAN_LEFT  : usize = 4;
pub const FAN_RIGHT : usize = 5;
pub const CANDLE    : usize = 6;

const BONUS         : Range<usize> = 37..41;
pub const PAPER     : usize = 0;
pub const CLOCK     : usize = 1;
pub const BANDS     : usize = 2;
pub const BATTERY   : usize = 3;

const DECOR         : Range<usize> = 41..49;
pub const TABLE     : usize = 0;
pub const SHREDDER  : usize = 1;
pub const BOOKS     : usize = 2;
pub const GUITAR    : usize = 3;
pub const SHELF     : usize = 4;
pub const BASKET    : usize = 5;
pub const PAINTING  : usize = 6;
pub const COMPUTER  : usize = 7;

const POWER         : Range<usize> = 49..55;
pub const SWITCH    : usize = 0;
pub const THERMO    : usize = 1;
pub const OUTLET    : usize = 2;
pub const SPARK     : Range<usize> = 3..5;
pub const TOGGLE    : usize = 5;

const TOASTS        : Range<usize> = 55..62;
pub const TOASTER   : usize = 0;
pub const TOAST     : Range<usize> = 1..7;

const GOLDFISH      : Range<usize> = 62..67;
pub const FISHBOWL  : usize = 0;
pub const FISH      : Range<usize> = 1..5;

const CANDLE_FLAME  : Range<usize> = 67..70;
pub const FLAME     : Range<usize> = 0..3;

const WATER_DRIP    : Range<usize> = 70..75;
pub const STILL_DRIP: usize = 0;
pub const DRIP      : Range<usize> = 0..5;

const BASKETBALL    : Range<usize> = 75..76;
pub const BALL      : usize = 0;

const GREASE        : Range<usize> = 76..79;
pub const UPRIGHT   : usize = 0;
pub const TIPPING   : usize = 1;
pub const SPILLED   : usize = 2;

const DART          : Range<usize> = 79..81;
pub const CRUSHED   : usize = 0;
pub const FLYING    : Range<usize> = 1..2;

const COPTER        : Range<usize> = 81..90;
pub const CRUMPLED  : usize = 0;
pub const FALLING   : Range<usize> = 1..9;

const BALLOON       : Range<usize> = 90..94;
pub const POPPED    : usize = 0;
pub const RISING    : Range<usize> = 1..4;

const RUBBER        : Range<usize> = 94..97;
pub const SHOT      : Range<usize> = 0..3;

const STAIRS        : Range<usize> = 97..99;
pub const STAIRS_UP     : usize = 0;
pub const STAIRS_DOWN   : usize = 1;

const SHREDDED      : Range<usize> = 99..100;
pub const BITS      : usize = 0;

pub struct Atlas<P> {
    pixels: P,
    blocks: HashMap<&'static str, &'static [Rect]>
}

impl<P> AsRef<P> for Atlas<P> {
    fn as_ref(&self) -> &P { &self.pixels }
}

impl<P> Index<&str> for Atlas<P> {
    type Output = [Rect];
    fn index(&self, index: &str) -> &Self::Output { &self.blocks[index] }
}

impl<P> Atlas<P> {
    pub fn get(&self, name: &str) -> (&'static [Rect], &P) {
        (self.blocks[name], &self.pixels)
    }
}

pub fn glider_sprites<P>(pixels: P) -> Atlas<P> {
    let blocks = HashMap::from_iter(
        [
            ("glider.right", &ITEMS[GLIDE_RIGHT]),
            ("glider.left", &ITEMS[GLIDE_LEFT]),
            ("glider.turn", &ITEMS[GLIDE_TURN]),
            ("glider.shreds", &ITEMS[SHREDDED]),

            ("blowers", &ITEMS[AIR]),
            ("collectible", &ITEMS[BONUS]),
            ("visual", &ITEMS[DECOR]),
            ("power", &ITEMS[POWER]),
            ("shock", &ITEMS[POWER][SPARK]),
            ("grease", &ITEMS[GREASE]),
            ("band", &ITEMS[RUBBER]),

            ("toaster", &ITEMS[TOASTS]),
            ("fish", &ITEMS[GOLDFISH]),
            ("fire", &ITEMS[CANDLE_FLAME]),
            ("water", &ITEMS[WATER_DRIP]),
            ("ball", &ITEMS[BASKETBALL]),
            ("dart", &ITEMS[DART]),
            ("copter", &ITEMS[COPTER]),
            ("balloon", &ITEMS[BALLOON]),
            ("stairs", &ITEMS[STAIRS]),
        ]
    );
    Atlas {
        pixels,
        blocks,
    }
}

pub fn rooms() -> HashMap<usize, Surface<'static>> {
    HashMap::from_iter(
        crate::resources::color::assets().iter().filter_map(
            |(&index, &bytes)| 
                (index >= 200).then(|| 
                    Some((index, {
                        let mut bits = Surface::new(room::SCREEN_WIDTH, room::SCREEN_HEIGHT, PixelFormatEnum::ABGR8888).ok()?.into_canvas().ok()?;
                        let processor = bits.texture_creator();
                        let pict = processor.load_texture_bytes(bytes).ok()?;
                        bits.copy(&pict, None, None).ok();
                        bits.into_surface()
                    }))
                )?
        )
    )
}
