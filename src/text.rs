pub fn process(text: &str, tab_width: usize) -> String {
    let spaced_text = text.replace('\t', &" ".repeat(tab_width));

    let trim_start = spaced_text.lines().fold(usize::MAX, |spaces, line| {
        spaces.min(line.bytes().position(|b| b != b' ').unwrap_or(line.len()))
    });

    spaced_text
        .lines()
        .map(|line| line[trim_start..].trim_end())
        .join_newline()
}

pub trait StrIterator: Iterator<Item = Self::StrItem> + Sized {
    type StrItem: AsRef<str>;

    fn join_newline(self) -> String {
        let mut s = self.fold(String::new(), |mut joined, line| {
            joined.push_str(line.as_ref());
            joined.push('\n');
            joined
        });
        s.pop();
        s
    }
}
impl<T, I> StrIterator for T
where
    T: Iterator<Item = I>,
    I: AsRef<str>,
{
    type StrItem = I;
}
