use super::FzfQuery;

/// TODO: docs
#[derive(Default, Clone)]
pub struct FzfParser {
    vec: Vec<char>,
}

impl FzfParser {
    /// TODO: docs
    #[inline]
    pub fn parse<'a>(&'a mut self, query: &str) -> FzfQuery<'a> {
        if query.len() > self.vec.len() {
            self.vec.resize(query.len(), '\0');
        }

        let mut char_len = 0;

        for ch in query.chars() {
            self.vec[char_len] = ch;
            char_len += 1;
        }

        FzfQuery::new(&self.vec[..char_len])
    }

    /// TODO: docs
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}
