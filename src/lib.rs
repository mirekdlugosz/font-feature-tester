mod constants;

use std::fs::File;

use anyhow::{Context as ErrorContext, Result};
use cairo::{Context, Format, ImageSurface};
use freetype::{Face as FTFace, face::LoadFlag, RenderMode};
use harfbuzz_rs::{UnicodeBuffer,
    Owned, Font as HBFont,
    Feature,
    Language,
    Direction,
    Tag,
    GlyphBuffer,
    GlyphInfo,
    GlyphPosition,
    shape as hb_shape
};

pub use constants::{HARFBUZZ_SCALING_FACTOR, SCREEN_DPI, BASE_SCREEN_DPI, DEFAULT_FONT_SIZE};

pub struct HBConfig<'a> {
    pub hb_font: Owned<HBFont<'a>>,
    pub font_features: &'a [Feature],
    pub direction: Direction,
    pub script: Tag,
    pub language: Language,
}

impl HBConfig<'_> {
    fn shape(hb_config: &Self, text: &str) -> GlyphBuffer {
        let mut hb_buffer = UnicodeBuffer::new();
        hb_buffer = hb_buffer.set_direction(hb_config.direction);
        hb_buffer = hb_buffer.set_script(hb_config.script);
        hb_buffer = hb_buffer.set_language(hb_config.language);
        hb_buffer = hb_buffer.add_str(text);
        hb_shape(&hb_config.hb_font, hb_buffer, hb_config.font_features)
    }
}

struct RasterizedGlyph {
    cr_is: ImageSurface,
    x_offset: f64,
    y_offset: f64,
}

pub fn draw_text(
    ft_face: FTFace,
    hb_config: &HBConfig,
    cr_context: Context,
    text: &[String],
    output: &mut File,
) -> Result<()> {
    let line_height = ft_face.size_metrics().map_or_else(
        || (DEFAULT_FONT_SIZE * 4 / 3) as f64,
        |metrics| metrics.y_ppem as f64
    );
    let line_advance = line_height * 1.5;
    let mut line_offset = line_advance;

    for line in text {
        // to mogłoby być ładniej
        if ! line.is_empty() {
            let shaped_text = HBConfig::shape(hb_config, line.as_str());

            draw_single_line(
                &ft_face,
                &cr_context,
                shaped_text.get_glyph_infos(),
                shaped_text.get_glyph_positions(),
                line_offset,
            )?;
        }
        line_offset += line_advance;
    }

    // Save the image as PNG
    cr_context.target().write_to_png(output)?;
    Ok(())
}

fn draw_single_line(
    ft_face: &FTFace,
    cr_context: &Context,
    glyph_infos: &[GlyphInfo],
    glyph_positions: &[GlyphPosition],
    line_offset: f64,
) -> Result<()> {
    let dpi_factor = SCREEN_DPI / BASE_SCREEN_DPI;
    let mut next_pos: f64 = 0.0;

    for (position, info) in glyph_positions.iter().zip(glyph_infos) {
        let rasterized = rasterize_glyph(ft_face, info.codepoint as u32)?;

        // We told FreeType to assume specific DPI, but HarfBuzz and Cairo do not know about it.
        // We could use Cairo set_device_scale, but then we would need to change bitmap_* values,
        // to avoid double-scaling. Also, device_scale seems to blurry the fonts in barely
        // noticeable way, but I may be imagining things.
        cr_context.mask_surface(
            rasterized.cr_is,
            next_pos + rasterized.x_offset,
            line_offset - ((position.y_advance as f64) / HARFBUZZ_SCALING_FACTOR * dpi_factor + rasterized.y_offset)
        )?;
        next_pos += position.x_advance as f64 / HARFBUZZ_SCALING_FACTOR * dpi_factor;
    }
    Ok(())
}

fn rasterize_glyph(
    ft_face: &FTFace,
    codepoint: u32,
) -> Result<RasterizedGlyph> {
        //let glyph_name = hb_font.get_glyph_name(info.codepoint as u32).unwrap_or("unknown".to_string());

        ft_face.load_glyph(codepoint, LoadFlag::DEFAULT)?;
        let ft_glyph = ft_face.glyph();
        ft_glyph.render_glyph(RenderMode::Normal)?;

        let bitmap = ft_glyph.bitmap();
        let bitmap_width = bitmap.width() as usize;
        let bitmap_height = bitmap.rows() as usize;
        let bitmap_row_length = bitmap.pitch() as usize;

        let bitmap_data = bitmap.buffer();

        let surface_row_length = Format::stride_for_width(Format::A8, bitmap_width as u32)?;
        let surface_size = bitmap_height.saturating_mul(surface_row_length as usize);
        let mut surface_data = vec![0; surface_size];

        for row_num in 0..bitmap_height {
            let bitmap_row_offset = row_num * bitmap_row_length;
            let surface_row_offset = row_num * surface_row_length as usize;
            for col_num in 0..bitmap_width {
                let bitmap_px_index = bitmap_row_offset + col_num;
                let surface_px_index = surface_row_offset + col_num;
                let value = bitmap_data
                    .get(bitmap_px_index)
                    .context("Failed to draw a glyph: bitmap index out of bounds")?;
                let svalue = surface_data
                    .get_mut(surface_px_index)
                    .context("Failed to draw a glyph: cairo surface index out of bounds")?;
                *svalue = *value;
            }
        }

        let cr_is = ImageSurface::create_for_data(
            surface_data, Format::A8, bitmap_width as i32, bitmap_height as i32, surface_row_length
        )?;

        Ok(RasterizedGlyph {
            cr_is,
            x_offset: ft_glyph.bitmap_left() as f64,
            y_offset: ft_glyph.bitmap_top() as f64,
        })
}
