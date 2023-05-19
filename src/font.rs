use std::slice;

use fontdue::{
    layout::{CoordinateSystem, GlyphPosition, Layout, LayoutSettings, TextStyle},
    Font, Metrics,
};

pub type Glyph = (GlyphPosition, Metrics, Vec<u8>);

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

pub fn rasterize(text: &str, font: &Font, size: usize) -> Vec<Glyph> {
    let mut layout: Layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x: 0.,
        y: 0.,
        max_width: None,
        max_height: None,
        line_height: 1.,
        ..Default::default()
    });

    layout.append(
        slice::from_ref(font),
        &TextStyle {
            text,
            px: size as f32,
            font_index: 0,
            user_data: (),
        },
    );

    layout
        .glyphs()
        .iter()
        .filter(|g| g.char_data.rasterize())
        .copied()
        .map(|g| {
            let (metrics, data) = font.rasterize_indexed(g.key.glyph_index, g.key.px);
            (g, metrics, data)
        })
        .collect()
}

pub fn dimensions(glyphs: &[Glyph]) -> Dimensions {
    glyphs.iter().fold(
        Dimensions {
            min_x: f32::MAX,
            min_y: f32::MAX,
            max_x: f32::MIN,
            max_y: f32::MIN,
        },
        |mut d, (g, ..)| {
            d.min_x = d.min_x.min(g.x);
            d.min_y = d.min_y.min(g.y);
            d.max_x = d.max_x.max(g.x + g.width as f32);
            d.max_y = d.max_y.max(g.y + g.height as f32);
            d
        },
    )
}

impl Dimensions {
    pub fn width(&self) -> usize {
        (self.max_x - self.min_x).ceil() as usize
    }

    pub fn height(&self) -> usize {
        (self.max_y - self.min_y).ceil() as usize
    }
}
