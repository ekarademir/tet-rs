use anyhow::Context;
use font::Glyph;

#[derive(Debug)]
pub struct Text {
    numbers: Vec<Glyph>,
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

        let mut numbers: Vec<Glyph> = Vec::new();
        let nums = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        for number in nums {
            let x = fonts[0]
                .draw(number)
                .context(format!("Can't draw {:?}", number))?;
            if let Some(n) = x {
                numbers.push(n);
            } else {
                return Err(anyhow::Error::msg(format!("Glyph is empty for {}", number)));
            }
        }

        Ok(Text { numbers })
    }
}
