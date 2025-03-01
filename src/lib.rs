mod constants;

use std::fs::File;
use std::str::FromStr;
use std::collections::HashMap;

use hex_color::{HexColor, ParseHexColorError};

use anyhow::{Context as ErrorContext, Result};
use cairo::{Context, Format, ImageSurface};
use freetype::{Face as FTFace, face::LoadFlag, RenderMode};
use harfbuzz_rs::{UnicodeBuffer,
    Owned, Font as HBFont,
    Face as HBFace,
    Feature,
    Language,
    Direction,
    Tag,
    GlyphBuffer,
    GlyphInfo,
    GlyphPosition,
    shape as hb_shape
};
use serde::Deserialize;

pub use constants::{HARFBUZZ_SCALING_FACTOR, SCREEN_DPI, BASE_SCREEN_DPI, DEFAULT_FONT_SIZE, DEFAULT_TEXT};

#[derive(Debug, PartialEq)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

impl FromStr for Color {
    type Err = ParseHexColorError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim_start_matches('#');
        let parsed = HexColor::parse(format!("#{s}").as_str())?;
        Ok(Color { 
            red: f64::from(parsed.r) / 255.0,
            green: f64::from(parsed.g) / 255.0,
            blue: f64::from(parsed.b) / 255.0,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct ConfigFile {
    pub font: FontConfig,
}

#[derive(Deserialize, Debug)]
pub struct FontConfig {
    pub file_path: String,
    pub size: Option<u32>,
    pub features: Option<HashMap<String, u32>>,
}

pub struct HBConfig<'a> {
    pub hb_font: Owned<HBFont<'a>>,
    pub font_features: Vec<Feature>,
    pub direction: Direction,
    pub script: Tag,
    pub language: Language,
}

impl HBConfig<'_> {
    pub fn create(font_path: &str, font_size: u32, feature_definitions: &Option<HashMap<String, u32>>) -> Result<Self> {
        let hb_face = HBFace::from_file(font_path, 0)
            .with_context(|| format!("Failed to open {font_path}"))?;
        let mut hb_font = HBFont::new(hb_face);
        hb_font.set_scale(
            font_size.saturating_mul(HARFBUZZ_SCALING_FACTOR as u32) as i32,
            font_size.saturating_mul(HARFBUZZ_SCALING_FACTOR as u32) as i32,
        );

        let features: Vec<Feature> = feature_definitions
            .as_ref()
            .map(|feats| feats
                .iter()
                .map(|(k, v)| hb_new_feature(k, *v))
                .collect()
            ).unwrap_or_default();

        Ok(HBConfig {
            hb_font,
            font_features: features,
            direction: Direction::Ltr,
            script: b"Latn".into(),
            language: Language::default(),
        })
    }

    fn shape(hb_config: &Self, text: &str) -> GlyphBuffer {
        let mut hb_buffer = UnicodeBuffer::new();
        hb_buffer = hb_buffer.set_direction(hb_config.direction);
        hb_buffer = hb_buffer.set_script(hb_config.script);
        hb_buffer = hb_buffer.set_language(hb_config.language);
        hb_buffer = hb_buffer.add_str(text);
        hb_shape(&hb_config.hb_font, hb_buffer, &hb_config.font_features)
    }
}

#[must_use] pub fn hb_new_feature(name: &str, value: u32) -> Feature {
    let mut tag = [0; 4];
    let bytes: Vec<u8> = name.bytes().take(4).collect();
    let bytes_len = bytes.len();
    tag[..bytes_len].copy_from_slice(&bytes);

    Feature::new(
        &tag,
        value,
        ..
    )
}

struct RasterizedGlyph {
    cr_is: ImageSurface,
    x_offset: f64,
    y_offset: f64,
}

pub fn get_text(input_file_path: Option<&str>) -> Vec<String> {
    input_file_path
        .map_or_else(
            || Ok(DEFAULT_TEXT.to_string()),
            std::fs::read_to_string
        )
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

    text
        .iter()
        .enumerate()
        .filter(|(_, line)| ! line.is_empty())
        .try_for_each(|(text_row, line)| {
            let shaped_text = HBConfig::shape(hb_config, line.as_str());
            let line_offset = text_row as f64 * line_advance + line_advance;
            draw_single_line(
                &ft_face,
                &cr_context,
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
            line_offset - (f64::from(position.y_advance) / HARFBUZZ_SCALING_FACTOR * dpi_factor + rasterized.y_offset)
        )?;
        next_pos += f64::from(position.x_advance) / HARFBUZZ_SCALING_FACTOR * dpi_factor;
    }
    Ok(())
}

fn rasterize_glyph(
    ft_face: &FTFace,
    codepoint: u32,
) -> Result<RasterizedGlyph> {
        ft_face.load_glyph(codepoint, LoadFlag::DEFAULT)?;
        let ft_glyph = ft_face.glyph();
        ft_glyph.render_glyph(RenderMode::Normal)?;

        let bitmap = ft_glyph.bitmap();
        let bitmap_width = bitmap.width();
        let bitmap_height = bitmap.rows();
        let bitmap_row_length = usize::try_from(bitmap.pitch())?;

        let bitmap_data = bitmap.buffer();

        let surface_row_length = Format::stride_for_width(Format::A8, u32::try_from(bitmap_width)?)?;
        let surface_size = usize::try_from(bitmap_height)?.saturating_mul(usize::try_from(surface_row_length)?);
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
            surface_data, Format::A8, bitmap_width, bitmap_height, surface_row_length
        )?;

        Ok(RasterizedGlyph {
            cr_is,
            x_offset: f64::from(ft_glyph.bitmap_left()),
            y_offset: f64::from(ft_glyph.bitmap_top()),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_no_hash() {
        let result = Color::from_str("1e1e2e");
        let expected = Color { red: 30.0 / 255.0, green: 30.0 / 255.0, blue: 46.0 / 255.0 };
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn parse_with_hash() {
        let result = Color::from_str("#179299");
        let expected = Color { red: 23.0 / 255.0, green: 146.0 / 255.0, blue: 153.0 / 255.0 };
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn feature_from_str() {
        let result = hb_new_feature("zero", 1);
        let expected = Feature::new(b"zero", 1, ..);
        assert_eq!(result.tag(), expected.tag());
        assert_eq!(result.value(), expected.value());
    }

    #[test]
    fn feature_from_short_str() {
        let result = hb_new_feature("ze", 1);
        let expected = Feature::new(b"ze\0\0", 1, ..);
        assert_eq!(result.tag(), expected.tag());
        assert_eq!(result.value(), expected.value());
    }

    #[test]
    fn feature_from_long_str() {
        let result = hb_new_feature("zerozero", 1);
        let expected = Feature::new(b"zero", 1, ..);
        assert_eq!(result.tag(), expected.tag());
        assert_eq!(result.value(), expected.value());
    }
}
