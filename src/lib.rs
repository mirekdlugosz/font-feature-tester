use std::fs::File;

use cairo::{Context, Format, ImageSurface};
use freetype::{Face as FTFace, face::LoadFlag, RenderMode};
use harfbuzz_rs::{UnicodeBuffer,
    Owned, Font as HBFont,
    Feature,
    Language,
    Direction,
    shape as hb_shape
};

pub fn draw_text(
    ft_face: FTFace,
    hb_font: Owned<HBFont>,
    cr_context: Context,
    text: &[String],
    output: &mut File,
) -> Result<(), ()> {
    let mut hb_buffer = UnicodeBuffer::new();
    hb_buffer = hb_buffer.set_direction(Direction::Ltr);
    hb_buffer = hb_buffer.set_script(b"Latn".into());
    hb_buffer = hb_buffer.set_language(Language::default());

    hb_buffer = hb_buffer.add_str(text.first().unwrap().as_str());

    let features = [
        Feature::new(
            b"cv14",
            1,
            ..
        ),
    ];

    let shaped_text = hb_shape(&hb_font, hb_buffer, &features);

    let glyph_infos = shaped_text.get_glyph_infos();
    let glyph_positions = shaped_text.get_glyph_positions();

    let mut next_pos: f64 = 0.0;

    for (position, info) in glyph_positions.iter().zip(glyph_infos) {
        //let glyph_name = hb_font.get_glyph_name(info.codepoint as u32).unwrap_or("unknown".to_string());

        let _ = ft_face.load_glyph(info.codepoint as u32, LoadFlag::DEFAULT);
        let ft_glyph = ft_face.glyph();
        let _ = ft_glyph.render_glyph(RenderMode::Normal);

        let bitmap = ft_glyph.bitmap();
        let bitmap_width = bitmap.width() as usize;
        let bitmap_height = bitmap.rows() as usize;
        let bitmap_row_length = bitmap.pitch() as usize;

        let bitmap_data = bitmap.buffer();

        let surface_row_length = Format::stride_for_width(Format::A8, bitmap_width as u32).unwrap();
        let surface_size = bitmap_height.saturating_mul(surface_row_length as usize);
        let mut surface_data = vec![0; surface_size];
        for row_num in 0..bitmap_height {
            let bitmap_row_offset = row_num * bitmap_row_length;
            let surface_row_offset = row_num * surface_row_length as usize;
            for col_num in 0..bitmap_width {
                let bitmap_px_index = bitmap_row_offset + col_num;
                let surface_px_index = surface_row_offset + col_num;
                let value = bitmap_data.get(bitmap_px_index).unwrap();
                let svalue = surface_data.get_mut(surface_px_index).unwrap();
                *svalue = *value;
                //if let Some(svalue) = surface_data.get_mut(surface_px_index) {
                    //*svalue = *value;
                //}
            }
        }

        cr_context.set_source_rgb(205.0 / 255.0, 214.0 / 255.0, 244.0 / 255.0);
        let cr_is = ImageSurface::create_for_data(
            surface_data, Format::A8, bitmap_width as i32, bitmap_height as i32, surface_row_length
        ).unwrap();
        let device_scale = cr_context.target().device_scale();
        cr_is.set_device_scale(device_scale.0, device_scale.1);
        // this value is also in main
        let target_dpi = 94.0;
        let _ = cr_context.mask_surface(
            cr_is,
            next_pos + ft_glyph.bitmap_left() as f64 * 72.0 / target_dpi,
            // 100 = row height
            // co to za magiczne 64.0? - to musi się zgadzać z wartością ustawioną w 
                // hb_font.set_scale(font_size as i32 * 64, font_size as i32 * 64);
            20.0 - ((position.y_advance as f64) / 64.0 + ft_glyph.bitmap_top() as f64 * 72.0 / target_dpi)
        );
        // patrz wyżej o magiczne 64
        next_pos += position.x_advance as f64 / 64.0;
    }

    // Save the image as PNG
    let _ = cr_context.target().write_to_png(output);
    Ok(())
}
