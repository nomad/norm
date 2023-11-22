/// TODO: docs
pub(crate) trait Opts: Copy {
    /// TODO: docs
    fn char_eq(&self, query_ch: char, candidate_ch: char) -> bool;

    /// TODO: docs
    fn find_first(
        &self,
        query_ch: char,
        candidate: &str,
    ) -> Option<(usize, usize)>;

    /// TODO: docs
    fn find_last(
        &self,
        query_ch: char,
        candidate: &str,
    ) -> Option<(usize, usize)>;

    /// TODO: docs
    fn to_char_offset(&self, candidate: &str, byte_offset: usize) -> usize;
}
