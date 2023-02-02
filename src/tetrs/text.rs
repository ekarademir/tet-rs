use anyhow::Context;

use wgpu_text::section::{BuiltInLineBreaker, Color, Layout, Section, Text, VerticalAlign};
use wgpu_text::BrushBuilder;

pub fn render(tetrs: &super::Tetrs, view: &wgpu::TextureView, text: &str) -> anyhow::Result<()> {
    let (left_margin, top_margin) = {
        (
            (tetrs.scene.window_size.width - tetrs.scene.scene_size.width) / 2,
            (tetrs.scene.window_size.height - tetrs.scene.scene_size.height) / 2,
        )
    };

    let font: &[u8] = include_bytes!("font.otf");
    let mut brush = BrushBuilder::using_font_bytes(font)
        .context("Can't parse font")?
        .build(&tetrs.base.device, &tetrs.base.surface_config);
    let font_size = 50.;
    let colour: Color = super::colours::YELLOW.into();
    let pos_x = (super::scene::LEFT_MARGIN + super::scene::GAME_AREA_WIDTH + super::scene::SPACE)
        * tetrs.scene.block_size
        + left_margin;
    let pos_y =
        (super::scene::TOP_MARGIN + super::scene::GAME_AREA_HEIGHT / 2 + super::scene::SPACE)
            * tetrs.scene.block_size
            + top_margin;
    let section = Section::default()
        .add_text(Text::new(text).with_scale(font_size).with_color(colour))
        .with_bounds((
            tetrs.base.surface_config.width as f32 / 2.0,
            tetrs.base.surface_config.height as f32,
        ))
        .with_layout(
            Layout::default()
                .v_align(VerticalAlign::Center)
                .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
        )
        .with_screen_position((pos_x as f32, pos_y as f32))
        .to_owned();

    let mut encoder = tetrs
        .base
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    // Render pass
    {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
    }
    brush.queue(&section);
    let cmd_buffer = brush.draw(&tetrs.base.device, &view, &tetrs.base.queue);
    tetrs.base.queue.submit([encoder.finish(), cmd_buffer]);
    Ok(())
}
