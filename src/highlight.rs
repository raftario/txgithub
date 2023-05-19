use std::ops::Range;

use rangemap::RangeMap;
use syntect::{
    highlighting::{HighlightState, Highlighter, RangedHighlightIterator, Style, Theme},
    parsing::{ParseState, ScopeStack, SyntaxSet},
    Error,
};

pub type StyleMap = RangeMap<usize, Style>;

pub fn highlight(
    text: &str,
    extension: &str,
    syntaxes: &SyntaxSet,
    theme: &Theme,
) -> Result<StyleMap, Error> {
    let syntax = syntaxes
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| syntaxes.find_syntax_plain_text());
    let mut parse_state = ParseState::new(syntax);

    let highlighter = Highlighter::new(theme);
    let mut highlight_state = HighlightState::new(&highlighter, ScopeStack::new());

    let mut offset = 0;
    let mut ranges = StyleMap::new();

    for line in text.lines() {
        let changes = parse_state.parse_line(line, syntaxes)?;
        let styles =
            RangedHighlightIterator::new(&mut highlight_state, &changes, line, &highlighter);

        for (style, _, Range { start, end }) in styles {
            ranges.insert((offset + start)..(offset + end), style);
        }

        offset += line.len() + 1;
    }

    Ok(ranges)
}
