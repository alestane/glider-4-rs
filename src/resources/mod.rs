
pub mod color {
    use std::collections::HashMap;
    const SPRITES: &[u8] = include_bytes!("color/128.png");
    const ROOM_200: &[u8] = include_bytes!("color/200.png");

    pub fn assets() -> HashMap<usize, &'static [u8]> {
        let mut values = HashMap::<usize, &'static [u8]>::new();
        values.insert(128, SPRITES);
        values.insert(200, ROOM_200);
        values
    }
}

pub const CIRCLE: &'static [u8] = include_bytes!("circle.raw");

pub const THE_HOUSE: &'static [u8] = include_bytes!("The House");