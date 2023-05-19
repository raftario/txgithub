use tokio::sync::{mpsc, oneshot};
use tracing::{Level, Span};

use crate::web::env::Env;

#[derive(Clone)]
pub struct Worker {
    tx: mpsc::Sender<Message>,
}

struct Message {
    text: String,
    extension: String,
    font: Option<String>,
    theme: Option<String>,
    tab_width: Option<usize>,
    tx: oneshot::Sender<Option<Vec<u8>>>,
    span: Span,
}

impl Worker {
    pub fn new(env: Env) -> Self {
        let (tx, rx) = mpsc::channel(4);
        tokio::task::spawn_blocking(move || Self::run(rx, env));
        Self { tx }
    }

    pub async fn render(
        &self,
        text: String,
        extension: String,
        font: Option<String>,
        theme: Option<String>,
        tab_width: Option<usize>,
    ) -> Option<Vec<u8>> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message {
                text,
                extension,
                font,
                theme,
                tab_width,
                tx,
                span: Span::current(),
            })
            .await
            .ok();
        rx.await.ok().flatten()
    }

    fn run(mut rx: mpsc::Receiver<Message>, env: Env) {
        let syntaxes = crate::vendor::syntaxes::load_syntaxes();
        let themes = crate::vendor::themes::load_themes();

        let default_theme = themes
            .themes
            .get(&env.default_theme)
            .expect("invalid default theme");
        let default_font = crate::vendor::fonts::load_font(&env.default_font, env.font_size)
            .expect("invalid default font");

        while let Some(Message {
            text,
            extension,
            font,
            theme,
            tab_width,
            tx,
            span,
        }) = rx.blocking_recv()
        {
            let span = tracing::span!(parent: span, Level::INFO, "render").entered();

            let font = font
                .and_then(|f| crate::vendor::fonts::load_font(&f, env.font_size))
                .unwrap_or(default_font);
            let theme = theme
                .and_then(|t| themes.themes.get(&t))
                .unwrap_or(default_theme);
            let tab_width = tab_width.unwrap_or(env.default_tab_width);

            let text = crate::text::process(&text, tab_width);
            let styles = match crate::highlight::highlight(&text, &extension, &syntaxes, theme) {
                Ok(h) => h,
                Err(e) => {
                    tracing::error!("{e}");
                    tx.send(None).ok();
                    continue;
                }
            };
            let glyphs = crate::font::rasterize(&text, font, env.font_size);

            let image = crate::image::draw(&glyphs, &styles, env.padding, theme);
            let png = match crate::image::encode(&image) {
                Ok(png) => Some(png),
                Err(e) => {
                    tracing::error!("{e}");
                    None
                }
            };

            span.exit();
            tx.send(png).ok();
        }
    }
}
