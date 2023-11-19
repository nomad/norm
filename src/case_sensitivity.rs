/// Case sensitivity modes for search.
///
/// This defines the different types of case sensitivity that can be used when
/// searching for the characters of a query in a candidate string.
#[derive(Copy, Clone, Debug, Default)]
pub enum CaseSensitivity {
    /// The search is case-sensitive. For a successful match the case of the
    /// letters in the candidate must exactly match the case of the letters in
    /// the query.
    Sensitive,

    /// The search is case-insensitive. In this mode, the case of letters is
    /// ignored, allowing for matches regardless of whether the letters in the
    /// query and candidate are upper or lower case.
    Insensitive,

    /// In this mode, the case-sensitivity of the search is determined
    /// dynamically based on the letters of the query. If the query contains
    /// one or more uppercase letters the search is treated as case-sensitive,
    /// otherwise it's case-insensitive.
    #[default]
    Smart,
}
