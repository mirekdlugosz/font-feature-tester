use std::fs::File;
use cairo::{Context, FontSlant, FontWeight, Format, ImageSurface, Glyph};

use harfbuzz_rs::Face;
use harfbuzz_rs::Font;
use harfbuzz_rs::shape;
use harfbuzz_rs::UnicodeBuffer;

fn main() {
    let surface = cairo::ImageSurface::create(Format::ARgb32, 600, 600)
        .expect("could not create cairo surface");
    let context = cairo::Context::new(&surface).expect("could not create cairo context");

    //let font_data = std::fs::read("/tmp/JetBrainsMono-Regular.ttf").expect("Can't read font file");
    let font_data = std::fs::read("/tmp/IntelOneMono-Regular.otf").expect("Can't read font file");
    let face = Face::new(font_data, 0);
    let font = Font::new(face);

    //let buffer = UnicodeBuffer::new().add_str("Hello, Harfbuzz!");
    let buffer = UnicodeBuffer::new().add('%' as u32, 0);

    let shaped_text = shape(&font, buffer, &[]);

    context.set_source_rgb(1.0, 1.0, 1.0);
    context.set_font_size(26.0);
    //context.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    context.move_to(100.0, 100.0);

    let mut cairo_glyphs = Vec::new();
    let positions = shaped_text.get_glyph_positions();
    let infos = shaped_text.get_glyph_infos();

    for (position, info) in positions.iter().zip(infos) {
        let glyph = Glyph::new(
            (info.codepoint as u32).into(),
            context.current_point().expect("no current point").0 + (position.x_offset as f64 / 64.0),
            context.current_point().expect("no current point").1 - (position.y_offset as f64 / 64.0),
        );
        println!("codepoint: {:?}", info.codepoint);
        println!("glyph name: {:?}", font.get_glyph_name(info.codepoint).unwrap());
        cairo_glyphs.push(glyph);
        context.rel_move_to(position.x_advance as f64 / 64.0, position.y_advance as f64 / 64.0);
    }
    context.show_glyphs(&cairo_glyphs).expect("could not show glyphs");

    //let _ = context.paint().expect("could not paint on surface");

    let mut file = File::create("/tmp/output.png").expect("Couldn’t create file.");
    surface.write_to_png(&mut file).expect("Couldn't write to file");
}
