#[derive(Clone, Copy, Debug)]
pub struct Colour {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

pub const ORANGE: Colour = Colour {
    red: 252,
    green: 115,
    blue: 0,
    alpha: 255,
};

pub const GREEN: Colour = Colour {
    red: 191,
    green: 219,
    blue: 56,
    alpha: 255,
};

pub const DARK_GREEN: Colour = Colour {
    red: 31,
    green: 138,
    blue: 112,
    alpha: 255,
};

pub const YELLOW: Colour = Colour {
    red: 252,
    green: 226,
    blue: 42,
    alpha: 255,
};

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
