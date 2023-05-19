use serde::Deserialize;
use tracing_subscriber::EnvFilter;

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct Env {
    pub port: u16,
    pub image_subdomain: String,
    pub font_size: usize,
    pub padding: usize,
    pub default_theme: String,
    pub default_font: String,
    pub default_tab_width: usize,
    pub max_size: usize,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            port: 8080,
            image_subdomain: "i".to_string(),
            font_size: 32,
            padding: 48,
            default_theme: "ayu-mirage".to_string(),
            default_font: "iosevka".to_string(),
            default_tab_width: 4,
            max_size: 16 * 1024,
        }
    }
}

pub fn parse() -> (Env, EnvFilter) {
    let env = envy::from_env().unwrap_or_default();
    let filter = EnvFilter::from_env("LOG");
    (env, filter)
}
