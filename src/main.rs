use std::fs::File;

use anyhow::{Context as ErrorContext, Result};
use clap::Parser;
use cairo::{Context, Format, ImageSurface};
use freetype::Library;
use harfbuzz_rs::{Font, Face, Feature, Direction, Language};

use font_feature_tester::draw_text;
use font_feature_tester::{HARFBUZZ_SCALING_FACTOR, SCREEN_DPI, DEFAULT_FONT_SIZE};
use font_feature_tester::HBConfig;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CliArgs {
    /// Path to font file to use
    #[arg(short, long)]
    font_path: String,

    /// Path to file with text to print
    #[arg(short = 'i', long = "input-path")]
    input_text_path: Option<String>,

    /// Path to output file. Should have PNG extension
    #[arg(short, long)]
    output_path: String,

    /// Output image width
    #[arg(long, default_value_t = 800)]
    image_width: i32,

    /// Output image height
    #[arg(long, default_value_t = 800)]
    image_height: i32,
}

fn main() -> Result<()> {
    // Read command line args
    // Read settings from file
    // Initialize freetype, harfbuzz, cairo context
    let args = CliArgs::parse();

    // Initialize Cairo
    // we should pass image size as CLI param
    let surface = ImageSurface::create(Format::ARgb32, args.image_width, args.image_height)
        .context("Could not create cario surface")?;
    let cr_context = Context::new(&surface)?;

    // bg color
    cr_context.set_source_rgb(30.0 / 255.0, 30.0 / 255.0, 46.0 / 255.0);
    cr_context.paint()?;
    // font color
    cr_context.set_source_rgb(205.0 / 255.0, 214.0 / 255.0, 244.0 / 255.0);

    // Initialize FreeType
    let ft_library = Library::init()?;
    let font_path = args.font_path.as_str();
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

    // to trzeba ładniej
    let text: Vec<String> = match args.input_text_path {
        Some(input_path) => {
            let file_content = std::fs::read_to_string(&input_path)
                .with_context(|| format!("Could not read {}", input_path))?;
            file_content.lines().map(String::from).collect()
        },
        None => [
            "$ sign :: => ===".to_string(),
            "wąska łódź VA {}".to_string(),
            "|B".to_string(),
            "i tak jak teraz nie ma to znaczenia".to_string(),
        ].into(),
    };

    let output_path = args.output_path.as_str();
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
