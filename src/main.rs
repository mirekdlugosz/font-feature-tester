use std::fs::File;

use anyhow::{Context as ErrorContext, Result};
use cairo::{Context, Format, ImageSurface};
use freetype::Library;
use harfbuzz_rs::{Font, Face, Feature, Direction, Language};

use font_feature_tester::draw_text;
use font_feature_tester::{HARFBUZZ_SCALING_FACTOR, SCREEN_DPI, DEFAULT_FONT_SIZE};
use font_feature_tester::HBConfig;

fn main() -> Result<()> {
    // Read command line args
    // Read settings from file
    // Initialize freetype, harfbuzz, cairo context

    // Initialize Cairo
    // we should pass image size as CLI param
    let surface = ImageSurface::create(Format::ARgb32, 800, 800)
        .context("Could not create cario surface")?;
    let cr_context = Context::new(&surface)?;

    // bg color
    cr_context.set_source_rgb(30.0 / 255.0, 30.0 / 255.0, 46.0 / 255.0);
    cr_context.paint()?;
    // font color
    cr_context.set_source_rgb(205.0 / 255.0, 214.0 / 255.0, 244.0 / 255.0);

    // Initialize FreeType
    //let font_path = "/tmp/AtkinsonHyperlegibleNext-Regular.otf";
    let font_path = "/tmp/JetBrainsMono-Regular.ttf";
    //let font_path = "/tmp/IntelOneMono-Medium.otf";
    let ft_library = Library::init()?;
    let ft_face = ft_library.new_face(font_path, 0)
        .with_context(|| format!("Failed to open {}", font_path))?;
    ft_face.set_char_size(
        0, (DEFAULT_FONT_SIZE as isize).saturating_mul(64),
        0, SCREEN_DPI as u32
    )?;

    let hb_face = Face::from_file(font_path, 0)
        .with_context(|| format!("Failed to open {}", font_path))?;

    let mut hb_font = Font::new(hb_face);
    hb_font.set_scale(
        DEFAULT_FONT_SIZE.saturating_mul(HARFBUZZ_SCALING_FACTOR as u32) as i32,
        DEFAULT_FONT_SIZE.saturating_mul(HARFBUZZ_SCALING_FACTOR as u32) as i32,
    );
    // we should get size_metrics from font and call hb_font.set_ppem()
    let features = [
        Feature::new(
            b"cv14",
            1,
            ..
        ),
    ];
    let hb_config = HBConfig {
        hb_font,
        font_features: &features,
        direction: Direction::Ltr,
        script: b"Latn".into(),
        language: Language::default(),
    };

    let text = [
        "$ sign :: => ===".to_string(),
        "wąska łódź VA {}".to_string(),
        "|B".to_string(),
        "i tak jak teraz nie ma to znaczenia".to_string(),
    ];

    let output_path = "/tmp/output.png";
    let mut file = File::create(output_path)
        .with_context(|| format!("Could not create {}", output_path))?;

    draw_text(
        ft_face,
        &hb_config,
        cr_context,
        &text,
        &mut file,
    )?;
    Ok(())
}
