#[derive(Clone, Copy, Debug, Default)]
pub struct Colour {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

const fn convert(value: u32) -> Colour {
    let alpha = 0xFF;
    let blue = (value % 0x100) as u8;
    let value = value / 0x100;
    let green = (value % 0x100) as u8;
    let value = value / 0x100;
    let red = (value % 0x100) as u8;
    Colour {
        red,
        green,
        blue,
        alpha,
    }
}

pub const BLACK: Colour = convert(0x000000);
pub const WHITE: Colour = convert(0xFFFFFF);
pub const ORANGE: Colour = convert(0xF2921D);
pub const GREEN: Colour = convert(0xBFDB38);
pub const DARK_GREEN: Colour = convert(0x00425A);
pub const YELLOW: Colour = convert(0xFCE22A);
pub const MAROON: Colour = convert(0xA61F69);
pub const LIGHT_PURPLE: Colour = convert(0xA084DC);

impl Into<[f32; 4]> for Colour {
    fn into(self) -> [f32; 4] {
        [
            self.red as f32 / 255.0,
            self.green as f32 / 255.0,
            self.blue as f32 / 255.0,
            self.alpha as f32 / 255.0,
        ]
    }
}