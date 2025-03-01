use std::fs::File;
use std::str::FromStr;

use anyhow::{Context as ErrorContext, Result};
use clap::Parser;
use cairo::{Context, Format, ImageSurface};
use freetype::Library;
use harfbuzz_rs::{Font, Face, Feature, Direction, Language};

use font_feature_tester::{draw_text, hb_new_feature};
use font_feature_tester::{HARFBUZZ_SCALING_FACTOR, SCREEN_DPI, DEFAULT_FONT_SIZE, DEFAULT_TEXT};
use font_feature_tester::HBConfig;
use font_feature_tester::Color;
use font_feature_tester::ConfigFile;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CliArgs {
    /// Path to text configuration file to use
    #[arg(short, long)]
    configuration_path: String,

    /// Path to file with text to print
    #[arg(short = 'i', long = "input-path")]
    input_text_path: Option<String>,

    /// Path to output file. Should have PNG extension
    #[arg(short, long)]
    output_path: String,

    /// Output image background color
    #[arg(long = "bg-color", default_value = "#eff1f5")]
    image_bg_color: String,

    /// Output image text color
    #[arg(long = "fg-color", default_value = "#4c4f69")]
    image_fg_color: String,

    /// Output image width
    #[arg(long, default_value_t = 800)]
    image_width: i32,

    /// Output image height
    #[arg(long, default_value_t = 800)]
    image_height: i32,
}

fn main() -> Result<()> {
    let args = CliArgs::parse();

    let config_path = args.configuration_path;
    let config_content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read {}", &config_path))?;
    let font_configuration: ConfigFile = toml::from_str(&config_content)
        .with_context(|| format!("Failed to parse configuration file {}", &config_path))?;

    let font_path = font_configuration.font.file_path.as_str();
    let font_size = font_configuration.font.size.unwrap_or(DEFAULT_FONT_SIZE);

    let bg_color = Color::from_str(args.image_bg_color.as_str())
        .context("Failed to parse --bg-color")?;
    let text_color = Color::from_str(args.image_fg_color.as_str())
        .context("Failed to parse --fg-color")?;

    // Initialize Cairo
    let surface = ImageSurface::create(Format::ARgb32, args.image_width, args.image_height)
        .context("Could not create cario surface")?;
    let cr_context = Context::new(&surface)?;

    cr_context.set_source_rgb(bg_color.red, bg_color.green, bg_color.blue);
    cr_context.paint()?;
    cr_context.set_source_rgb(text_color.red, text_color.green, text_color.blue);

    // Initialize FreeType
    let ft_library = Library::init()?;
    let ft_face = ft_library.new_face(font_path, 0)
        .with_context(|| format!("Failed to open {}", font_path))?;
    ft_face.set_char_size(
        0, (font_size as isize).saturating_mul(64),
        0, SCREEN_DPI as u32
    )?;

    // Initialize HarfBuzz
    let hb_face = Face::from_file(font_path, 0)
        .with_context(|| format!("Failed to open {}", font_path))?;
    let mut hb_font = Font::new(hb_face);
    hb_font.set_scale(
        font_size.saturating_mul(HARFBUZZ_SCALING_FACTOR as u32) as i32,
        font_size.saturating_mul(HARFBUZZ_SCALING_FACTOR as u32) as i32,
    );
    let features: Vec<Feature> = font_configuration.font.features
        .map(|feats| feats
            .iter()
            .map(|(k, v)| hb_new_feature(k, *v))
            .collect()
        ).unwrap_or(vec!());
    let hb_config = HBConfig {
        hb_font,
        font_features: &features,
        direction: Direction::Ltr,
        script: b"Latn".into(),
        language: Language::default(),
    };

    let text: Vec<String> = args
        .input_text_path
        .map_or_else(
            || Ok(DEFAULT_TEXT.to_string()),
            std::fs::read_to_string
        )
        .unwrap_or_else(|e| {
            println!("Failed to read input file: {}. Falling back to default text", e);
            DEFAULT_TEXT.to_string()
        })
        .lines()
        .map(String::from)
        .collect();

    let output_path = args.output_path.as_str();
    let mut file = File::create(output_path)
        .with_context(|| format!("Could not create {}", output_path))?;

    draw_text(
        ft_face,
        font_size,
        &hb_config,
        cr_context,
        &text,
        &mut file,
    )?;
    Ok(())
}
