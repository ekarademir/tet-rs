use std::collections::HashMap;

use anyhow::Context;
use font::{Glyph, Segment};
use lyon::{math::point, path::Path};

const NUMS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

#[derive(Debug)]
pub struct Text {
    lookup: HashMap<char, Glyph>,
}

impl Text {
    pub fn new() -> anyhow::Result<Self> {
        let path = "./assets/font.otf";
        let font::File { mut fonts } = font::File::open(path).context(format!(
            "Can't find {:?} in {:?}",
            path,
            std::env::current_dir().unwrap()
        ))?;

        if fonts.len() == 0 {
            return Err(anyhow::Error::msg(format!("No fonts found in {}", path)));
        }
        log::debug!("{:?} fonts in {:?}", fonts.len(), path);

        let mut lookup: HashMap<char, Glyph> = HashMap::new();

        for number in NUMS {
            let x = fonts[0]
                .draw(number)
                .context(format!("Can't draw {:?}", number))?;
            if let Some(n) = x {
                lookup.insert(number, n);
            } else {
                return Err(anyhow::Error::msg(format!("Glyph is empty for {}", number)));
            }
        }

        Ok(Text { lookup })
    }

    pub fn write(&self, input: &str, colour: super::colours::Colour) -> super::drawable::Geometry {
        for c in input.chars() {
            if let Some(glyph) = self.lookup.get(&c) {
                let mut builder = Path::builder();
                builder.begin(point(0.0, 0.0));
                for contour in glyph.iter() {
                    for segment in contour.iter() {
                        match segment {
                            &Segment::Cubic(a, b, c) => {
                                builder.cubic_bezier_to(
                                    point(a.0, a.1),
                                    point(b.0, b.1),
                                    point(c.0, c.1),
                                );
                            }
                            &Segment::Quadratic(a, b) => {
                                builder.quadratic_bezier_to(point(a.0, a.1), point(b.0, b.1));
                            }
                            &Segment::Linear(a) => {
                                builder.line_to(point(a.0, a.1));
                            }
                        }
                    }
                }
                builder.close();

                let path = builder.build();
            }
        }
        todo!()
    }
}
