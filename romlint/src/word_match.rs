const IGNORED_WORDS: [&str; 3] = ["a", "of", "the"];

pub struct Tokens<'a> {
    words: Vec<&'a str>,
}

impl<'a> Tokens<'a> {
    pub fn from_str(s: &'a str) -> Self {
        let tokens = match s.rsplit_once('.') {
            Some((name, _ext)) => name,
            None => s,
        };
        let tokens = tokens.split_whitespace();
        let (_tags, words) = tokens.partition(|&token| {
            token.starts_with('(') && token.ends_with(')')
                || token.starts_with('[') && token.ends_with(']')
        });

        Self { words }
    }

    pub fn words_in_common_with(&self, other: &Tokens) -> usize {
        self.words
            .iter()
            .filter(|word| !IGNORED_WORDS.contains(word))
            .filter(|word| other.words.contains(word))
            .count()
    }

    pub fn word_count(&self) -> usize {
        self.words.len()
    }
}
