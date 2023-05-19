use std::io::Cursor;

use image::{
    GenericImage, GenericImageView, ImageError, ImageOutputFormat, Pixel, Rgba, RgbaImage,
};
use syntect::highlighting::{Color, Theme};

use crate::{font::Glyph, highlight::StyleMap};

fn ctp(color: Color) -> Rgba<u8> {
    Rgba([color.r, color.g, color.b, color.a])
}

pub fn draw(glyphs: &[Glyph], styles: &StyleMap, padding: usize, theme: &Theme) -> RgbaImage {
    let dimensions = crate::font::dimensions(glyphs);
    let width = dimensions.width() + padding * 2;
    let height = dimensions.height() + padding * 2;

    let mut image = RgbaImage::from_pixel(
        width as u32,
        height as u32,
        ctp(theme.settings.background.unwrap()),
    );

    for (glyph_position, _, glyph_data) in glyphs {
        let color = ctp(styles
            .get(&glyph_position.byte_offset)
            .map(|s| s.foreground)
            .unwrap_or(theme.settings.foreground.unwrap()));

        let width = glyph_position.width as u32;
        let height = glyph_position.height as u32;
        let mut subimage = image.sub_image(
            (padding as f32 + glyph_position.x - dimensions.min_x) as u32,
            (padding as f32 + glyph_position.y - dimensions.min_y) as u32,
            width,
            height,
        );

        for x in 0..width {
            for y in 0..height {
                let alpha = glyph_data[(x + y * width) as usize];
                let pixel_color = color.map_with_alpha(
                    |c| c,
                    |a| (a as f32 * (alpha as f32 / u8::MAX as f32)) as u8,
                );

                let mut pixel = subimage.get_pixel(x, y);
                pixel.blend(&pixel_color);
                subimage.put_pixel(x, y, pixel);
            }
        }
    }

    image
}

pub fn encode(image: &RgbaImage) -> Result<Vec<u8>, ImageError> {
    let mut buf = Cursor::new(Vec::new());
    image.write_to(&mut buf, ImageOutputFormat::Png)?;
    Ok(buf.into_inner())
}
