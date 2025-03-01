use std::fs::File;

use cairo::{Context, Format, ImageSurface};
use freetype::Library;
use harfbuzz_rs::{Font, Face};

use font_feature_tester::draw_text;
use font_feature_tester::{HARFBUZZ_SCALING_FACTOR, SCREEN_DPI, DEFAULT_FONT_SIZE};

fn main() {
    // Read command line args
    // Read settings from file
    // Initialize freetype, harfbuzz, cairo context

    // Initialize Cairo
    // we should pass image size as CLI param
    let surface = ImageSurface::create(Format::ARgb32, 800, 800).unwrap();
    let cr_context = Context::new(&surface).unwrap();

    // bg color
    cr_context.set_source_rgb(30.0 / 255.0, 30.0 / 255.0, 46.0 / 255.0);
    let _ = cr_context.paint().unwrap();
    // font color
    cr_context.set_source_rgb(205.0 / 255.0, 214.0 / 255.0, 244.0 / 255.0);

    // Initialize FreeType
    //let font_path = "/tmp/AtkinsonHyperlegibleNext-Regular.otf";
    let font_path = "/tmp/JetBrainsMono-Regular.ttf";
    //let font_path = "/tmp/IntelOneMono-Medium.otf";
    let ft_library = Library::init().unwrap();
    let ft_face = ft_library.new_face(font_path, 0).unwrap();
    ft_face.set_char_size(
        0, (DEFAULT_FONT_SIZE as isize).saturating_mul(64),
        0, SCREEN_DPI as u32
    ).unwrap();

    let font_data = std::fs::read(font_path).expect("Can't read font file");
    let hb_face = Face::new(font_data, 0);

    let mut hb_font = Font::new(hb_face);
    hb_font.set_scale(
        DEFAULT_FONT_SIZE.saturating_mul(HARFBUZZ_SCALING_FACTOR as u32) as i32,
        DEFAULT_FONT_SIZE.saturating_mul(HARFBUZZ_SCALING_FACTOR as u32) as i32,
    );
    // we should get size_metrics from font and call hb_font.set_ppem()

    let text = [
        "$ sign :: => ===".to_string(),
        "wąska łódź VA {}".to_string(),
        "|B".to_string(),
        "i tak jak teraz nie ma to znaczenia".to_string(),
    ];

    let mut file = File::create("/tmp/output.png").expect("Couldn't create file.");

    let res = draw_text(
        ft_face,
        hb_font,
        cr_context,
        &text,
        &mut file,
    );

    match res {
        Ok(_) => (),
        Err(_) => {
            println!("Something went wrong!");
        },
    }
}
