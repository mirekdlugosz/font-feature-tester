use std::fs::File;
extern crate cairo;
extern crate freetype;
extern crate harfbuzz_rs as harfbuzz;

use image::{Luma, ImageBuffer};
use cairo::ImageSurface;
use cairo::{Context, Format};
use freetype::Library;
use harfbuzz_rs::{UnicodeBuffer, Font, Face, shape};

fn main() {
    // Initialize FreeType
    let ft_library = Library::init().unwrap();
    let ft_face = ft_library.new_face("/tmp/JetBrainsMono-Regular.ttf", 0).unwrap();
    //ft_face.set_char_size(48 * 64, 0, 0, 0).unwrap();
    let font_data = std::fs::read("/tmp/JetBrainsMono-Regular.ttf").expect("Can't read font file");
    let hb_face = Face::new(font_data, 0);

    // Create a HarfBuzz font
    let hb_font = Font::new(hb_face);

    // Create a buffer and add codepoint
    let mut buffer = UnicodeBuffer::new().add_str("$ sign ::");
    //let mut buffer = UnicodeBuffer::new().add_str("$");

    // Set text direction and script
    buffer = buffer.set_direction(harfbuzz::Direction::Ltr);
    //buffer.set_script(harfbuzz::Script::Latin);
    //buffer.set_language(harfbuzz::Language::from_string("en"));
    let features = [harfbuzz_rs::Feature::new(
        harfbuzz_rs::Tag::new('c', 'v', '1', '4'),
        1,
        ..
    )];


    // Shape the text
    let shaped_text = shape(&hb_font, buffer, &features);

    // Extract glyph information
    let glyph_infos = shaped_text.get_glyph_infos();
    let glyph_positions = shaped_text.get_glyph_positions();

    // Initialize Cairo
    let surface = ImageSurface::create(Format::ARgb32, 600, 600).unwrap();
    let cr = Context::new(&surface).unwrap();
    //cr.move_to(100.0, 100.0);
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.paint().unwrap();
    cr.set_source_rgb(0.0, 0.0, 0.0);

    // Render the glyph
    for (position, info) in glyph_positions.iter().zip(glyph_infos) {
        //let glyph = Glyph::new(
            //(info.codepoint as u32).into(),
            //context.current_point().expect("no current point").0 + (position.x_offset as f64 / 64.0),
            //context.current_point().expect("no current point").1 - (position.y_offset as f64 / 64.0),
        //);
        ft_face.load_glyph(info.codepoint as u32, freetype::face::LoadFlag::RENDER).unwrap();
        let ft_glyph = ft_face.glyph();
        let bitmap = ft_glyph.bitmap();
        let width = bitmap.width() as usize;
        let height = bitmap.rows() as usize;
        let data = bitmap.buffer();
        let size = (width * height * 4) as usize;
        let pitch = bitmap.pitch() as usize;
        let mut rgba = vec![0u8; size];
        let mut image_buffer = ImageBuffer::new(width as u32, height as u32);
        for y in 0..height {
            let src_offset = y;// * pitch as usize;
            let dest_offset = y * width * 4;
            for x in 0..width {
                let red = data[src_offset + (x * 3)];
                let green = data[src_offset + (x * 3) + 1];
                let blue = data[src_offset + (x * 3) + 2];
                //let alpha = red.min(green).min(blue);
                let alpha = 1.0 as u8;
                rgba[dest_offset + (x * 4)] = red;
                rgba[dest_offset + (x * 4) + 1] = green;
                rgba[dest_offset + (x * 4) + 2] = blue;
                rgba[dest_offset + (x * 4) + 3] = alpha;
                //image_buffer.put_pixel(x as u32, y as u32, image::Rgba([red, green, blue, alpha]));
                let value = bitmap.buffer()[(y * width + x) as usize];
                image_buffer.put_pixel(x as u32, y as u32, Luma([value]));
                cr.set_source_rgba(red as f64, green as f64, blue as f64, alpha as f64);
                //cr.rel_move_to(x as f64, y as f64);
                cr.rectangle(x as f64, y as f64, 1.5, 1.5);
                //cr.set_source_rgba(1.0, 0.0, 0.0, 1.0);
                cr.fill();
                //println!("x {:?} y {:?}", x, y);
            }
        }
        let glyph_name = hb_font.get_glyph_name(info.codepoint as u32).unwrap_or("unknown".to_string());
        let px_mode = bitmap.pixel_mode().unwrap();

        image_buffer.save(format!("/tmp/output-{}.png", glyph_name));

        let msg = match ft_glyph.outline() {
            Some(_) => "has glyph outline",
            None => "no glyph outline for you",
        };

        println!("glyph {} has pixel mode {:?} and {}", glyph_name, px_mode, msg);
        cr.move_to(100.0, 100.0);
        cr.rel_move_to(position.x_advance as f64 / 64.0, position.y_advance as f64 / 64.0);
        cr.close_path();
        //println!("Processing {:?}, cario point is {:?}", info.codepoint, cr.current_point().unwrap());

        //println!("codepoint: {:?} data: {:?}", info.codepoint, rgba);
        //cairo_glyphs.push(glyph);
        //context.rel_move_to(position.x_advance as f64 / 64.0, position.y_advance as f64 / 64.0);
    }

    // Save the image as PNG
    //let mut file = File::create("/tmp/output.png").expect("Couldn’t create file.");
    //surface.write_to_png(&mut file).expect("Couldn't write to file");
}
