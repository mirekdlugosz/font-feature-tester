use std::fs::File;

use anyhow::{Context as ErrorContext, Result};
use cairo::{Context, Format, ImageSurface};
use freetype::{Face as FTFace, RenderMode, face::LoadFlag};
use harfbuzz_rs::{GlyphInfo, GlyphPosition};

use constants::{BASE_SCREEN_DPI, DEFAULT_TEXT, HARFBUZZ_SCALING_FACTOR};

pub use crate::colors::Color;
pub use crate::hb::HBConfig;
pub use constants::{DEFAULT_FONT_SIZE, SCREEN_DPI};

mod colors;
mod constants;
mod hb;

struct RasterizedGlyph {
    cr_is: ImageSurface,
    x_offset: f64,
    y_offset: f64,
}

pub fn get_text(input_file_path: Option<&str>) -> Vec<String> {
    input_file_path
        .map_or_else(|| Ok(DEFAULT_TEXT.to_string()), std::fs::read_to_string)
        .unwrap_or_else(|e| {
            println!("Failed to read input file: {e}. Falling back to default text");
            DEFAULT_TEXT.to_string()
        })
        .lines()
        .map(String::from)
        .collect()
}

pub fn draw_text(
    ft_face: &FTFace,
    hb_config: &HBConfig,
    cr_context: &Context,
    line_height: f64,
    text: &[String],
    output: &mut File,
) -> Result<()> {
    let line_advance = line_height * 1.5;

    text.iter()
        .enumerate()
        .filter(|(_, line)| !line.is_empty())
        .try_for_each(|(text_row, line)| {
            let shaped_text = HBConfig::shape(hb_config, line.as_str());
            let line_offset = text_row as f64 * line_advance + line_advance;
            draw_single_line(
                ft_face,
                cr_context,
                shaped_text.get_glyph_infos(),
                shaped_text.get_glyph_positions(),
                line_offset,
            )
        })?;

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
        let rasterized = rasterize_glyph(ft_face, info.codepoint)?;

        // We told FreeType to assume specific DPI, but HarfBuzz and Cairo do not know about it.
        // We could use Cairo set_device_scale, but then we would need to change bitmap_* values,
        // to avoid double-scaling. Also, device_scale seems to blurry the fonts in barely
        // noticeable way, but I may be imagining things.
        cr_context.mask_surface(
            rasterized.cr_is,
            next_pos + rasterized.x_offset,
            line_offset
                - (f64::from(position.y_advance) / HARFBUZZ_SCALING_FACTOR * dpi_factor
                    + rasterized.y_offset),
        )?;
        next_pos += f64::from(position.x_advance) / HARFBUZZ_SCALING_FACTOR * dpi_factor;
    }
    Ok(())
}

fn rasterize_glyph(ft_face: &FTFace, codepoint: u32) -> Result<RasterizedGlyph> {
    ft_face.load_glyph(codepoint, LoadFlag::DEFAULT)?;
    let ft_glyph = ft_face.glyph();
    ft_glyph.render_glyph(RenderMode::Normal)?;

    let bitmap = ft_glyph.bitmap();
    let bitmap_width = bitmap.width();
    let bitmap_height = bitmap.rows();
    let bitmap_row_length = usize::try_from(bitmap.pitch())?;

    let bitmap_data = bitmap.buffer();

    let surface_row_length = Format::stride_for_width(Format::A8, u32::try_from(bitmap_width)?)?;
    let surface_size =
        usize::try_from(bitmap_height)?.saturating_mul(usize::try_from(surface_row_length)?);
    let mut surface_data = vec![0; surface_size];

    for row_num in 0..usize::try_from(bitmap_height)? {
        let bitmap_row_offset = row_num * bitmap_row_length;
        let surface_row_offset = row_num * usize::try_from(surface_row_length)?;
        for col_num in 0..usize::try_from(bitmap_width)? {
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
        surface_data,
        Format::A8,
        bitmap_width,
        bitmap_height,
        surface_row_length,
    )?;

    Ok(RasterizedGlyph {
        cr_is,
        x_offset: f64::from(ft_glyph.bitmap_left()),
        y_offset: f64::from(ft_glyph.bitmap_top()),
    })
}
