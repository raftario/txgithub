use syntect::parsing::SyntaxSet;

static SYNTAXES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/syntaxes"));

pub fn load_syntaxes() -> SyntaxSet {
    syntect::dumps::from_binary(SYNTAXES)
}
