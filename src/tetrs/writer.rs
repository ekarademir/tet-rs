use anyhow::Context;

use glyph_brush::ab_glyph::FontArc;
use wgpu_text::BrushBuilder;

pub struct Writer {
    pub brush: wgpu_text::TextBrush,
}

impl Writer {
    pub fn new(base: &super::Base) -> anyhow::Result<Writer> {
        let font =
            FontArc::try_from_slice(include_bytes!("font.otf")).context("Can't read font file")?;
        let brush = BrushBuilder::using_font(font).build(&base.device, &base.surface_config);
        Ok(Writer { brush })
    }
}
