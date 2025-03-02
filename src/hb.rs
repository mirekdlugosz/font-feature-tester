use std::collections::HashMap;

use crate::HARFBUZZ_SCALING_FACTOR;

use anyhow::{Context as ErrorContext, Result};
use harfbuzz_rs::{
    Direction, Face as HBFace, Feature, Font as HBFont, GlyphBuffer, Language, Owned, Tag,
    UnicodeBuffer, shape as hb_shape,
};

pub struct HBConfig<'a> {
    pub hb_font: Owned<HBFont<'a>>,
    pub font_features: Vec<Feature>,
    pub direction: Direction,
    pub script: Tag,
    pub language: Language,
}

impl HBConfig<'_> {
    pub fn create(
        font_path: &str,
        font_size: u32,
        feature_definitions: &Option<HashMap<String, u32>>,
    ) -> Result<Self> {
        let hb_face = HBFace::from_file(font_path, 0)
            .with_context(|| format!("Failed to open {font_path}"))?;
        let mut hb_font = HBFont::new(hb_face);
        hb_font.set_scale(
            font_size.saturating_mul(HARFBUZZ_SCALING_FACTOR as u32) as i32,
            font_size.saturating_mul(HARFBUZZ_SCALING_FACTOR as u32) as i32,
        );

        let features: Vec<Feature> = feature_definitions
            .as_ref()
            .map(|feats| feats.iter().map(|(k, v)| hb_new_feature(k, *v)).collect())
            .unwrap_or_default();

        Ok(HBConfig {
            hb_font,
            font_features: features,
            direction: Direction::Ltr,
            script: b"Latn".into(),
            language: Language::default(),
        })
    }

    #[must_use]
    pub fn shape(hb_config: &Self, text: &str) -> GlyphBuffer {
        let mut hb_buffer = UnicodeBuffer::new();
        hb_buffer = hb_buffer.set_direction(hb_config.direction);
        hb_buffer = hb_buffer.set_script(hb_config.script);
        hb_buffer = hb_buffer.set_language(hb_config.language);
        hb_buffer = hb_buffer.add_str(text);
        hb_shape(&hb_config.hb_font, hb_buffer, &hb_config.font_features)
    }
}

#[must_use]
pub fn hb_new_feature(name: &str, value: u32) -> Feature {
    let mut tag = [0; 4];
    let bytes: Vec<u8> = name.bytes().take(4).collect();
    let bytes_len = bytes.len();
    tag[..bytes_len].copy_from_slice(&bytes);

    Feature::new(&tag, value, ..)
}

#[cfg(test)]
mod tests {
    use super::*;

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
