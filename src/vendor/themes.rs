use syntect::highlighting::ThemeSet;

static THEMES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/themes"));

pub fn load_themes() -> ThemeSet {
    syntect::dumps::from_binary(THEMES)
}
