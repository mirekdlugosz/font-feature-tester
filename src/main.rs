use std::fs::File;

use cairo::{Context, Format, ImageSurface};
use freetype::Library;
use harfbuzz_rs::{Font, Face};

use font_feature_tester::draw_text;

fn main() {
    // Read command line args
    // Read settings from file
    // Initialize freetype, harfbuzz, cairo context

    // Initialize Cairo
    // we should pass that as param
    let target_dpi = 94.0;
    let surface = ImageSurface::create(Format::ARgb32, 800, 800).unwrap();
    surface.set_device_scale(target_dpi / 72.0, target_dpi / 72.0);
    let cr_context = Context::new(&surface).unwrap();
    cr_context.set_source_rgb(30.0 / 255.0, 30.0 / 255.0, 46.0 / 255.0);
    cr_context.paint().unwrap();

    // Initialize FreeType
    //let font_path = "/tmp/AtkinsonHyperlegibleNext-Regular.otf";
    //let font_path = "/tmp/JetBrainsMono-Regular.ttf";
    let font_path = "/tmp/IntelOneMono-Medium.otf";
    let ft_library = Library::init().unwrap();
    let ft_face = ft_library.new_face(font_path, 0).unwrap();
    let font_size = 15;
    ft_face.set_char_size(0, font_size * 64, 0, target_dpi as u32).unwrap();
    println!("{:?}", ft_face.size_metrics().unwrap());

    let font_data = std::fs::read(font_path).expect("Can't read font file");
    let hb_face = Face::new(font_data, 0);

    // Create a HarfBuzz font
    let mut hb_font = Font::new(hb_face);
    // dlaczego 64?
    hb_font.set_scale(font_size as i32 * 64, font_size as i32 * 64);
    hb_font.set_ppem(font_size as u32, font_size as u32);

    let text = [
        "wąska łódź VA {}".to_string(),
    ];
    //let mut buffer = UnicodeBuffer::new().add_str("$ sign :: => ===");
    //let mut buffer = UnicodeBuffer::new().add_str("|B");
    //let mut buffer = UnicodeBuffer::new().add_str("$");

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
