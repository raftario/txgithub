use std::{env, fs, path::Path};

use bat::assets::HighlightingAssets;
use quote::quote;
use syntect::highlighting::ThemeSet;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    syntaxes(out_dir);
    themes(out_dir);
    fonts(out_dir);
}

fn syntaxes(out_dir: &Path) {
    let assets = HighlightingAssets::from_binary();
    let set = assets.get_syntax_set().unwrap();
    let path = out_dir.join("syntaxes");
    syntect::dumps::dump_to_file(&set, path).unwrap();
}

fn themes(out_dir: &Path) {
    println!("cargo:rerun-if-changed=./vendor/themes");

    let set = ThemeSet::load_from_folder("./vendor/themes").unwrap();
    let path = out_dir.join("themes");
    syntect::dumps::dump_to_file(&set, path).unwrap();
}

fn fonts(out_dir: &Path) {
    println!("cargo:rerun-if-changed=./vendor/fonts");

    let fonts = fs::read_dir("./vendor/fonts")
        .unwrap()
        .filter_map(|r| r.ok())
        .filter(|e| e.path().extension().map_or(false, |e| e == "ttf"))
        .map(|font| {
            let path = font.path();
            let name = path.file_stem().unwrap().to_str().unwrap().to_string();

            quote! {
                #name => {
                    static DATA: &[u8] = include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"), "/vendor/fonts/", #name, ".ttf"
                    ));
                    static CELL: ::once_cell::sync::OnceCell<::fontdue::Font> =
                        ::once_cell::sync::OnceCell::new();
                    Some(CELL.get_or_init(|| {
                        ::fontdue::Font::from_bytes(DATA, settings).unwrap()
                    }))
                }
            }
        });

    let func = quote! {
        pub fn load_font(name: &str, size: usize) -> Option<&'static ::fontdue::Font> {
            let settings = ::fontdue::FontSettings {
                collection_index: 0,
                scale: size as f32,
            };
            let name = name.to_lowercase().replace(' ', "-");

            match name.as_str() {
                #(#fonts,)*
                _ => None,
            }
        }
    };

    let data = func.to_string();
    let path = out_dir.join("fonts.rs");
    fs::write(path, data).unwrap();
}
