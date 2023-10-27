#![allow(clippy::single_range_in_vec_init)]

use norm::fzf::{bonus, penalty};
use norm::CaseSensitivity;
use CaseSensitivity::*;

pub fn empty_query<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "", "foo");
    assert!(m.is_none());
}

pub fn upstream_1<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "oBZ", "fooBarbaz1");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH + bonus::CAMEL_123
            - penalty::GAP_START
            - 3 * penalty::GAP_EXTENSION
    );

    assert_eq!(m.matched_ranges().sorted(), [2..4, 8..9]);
}

pub fn upstream_2<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "fbb", "foo bar baz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH
            + bonus::FIRST_QUERY_CHAR_MULTIPLIER
                * fzf.scheme().bonus_boundary_white
            + 2 * fzf.scheme().bonus_boundary_white
            - 2 * penalty::GAP_START * 2
            - 4 * penalty::GAP_EXTENSION
    );

    assert_eq!(m.matched_ranges().sorted(), [0..1, 4..5, 8..9]);
}

pub fn upstream_3<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "rdoc", "/AutomatorDocument.icns");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        4 * bonus::MATCH + 2 * bonus::CONSECUTIVE + bonus::CAMEL_123
    );

    assert_eq!(m.matched_ranges().sorted(), [9..13]);
}

pub fn upstream_4<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "zhsc", "/man1/zshcompctl.1");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        4 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 3)
                * fzf.scheme().bonus_boundary_delimiter
    );

    assert_eq!(m.matched_ranges().sorted(), [6..10]);
}

pub fn upstream_5<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "zhsc", "/.oh-my-zsh/cache");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        4 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 2) * bonus::BOUNDARY
            + fzf.scheme().bonus_boundary_delimiter
            - penalty::GAP_START
    );

    assert_eq!(m.matched_ranges().sorted(), [8..11, 12..13]);
}

pub fn upstream_6<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "12356", "ab0123 456");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        5 * bonus::MATCH + 3 * bonus::CONSECUTIVE
            - penalty::GAP_START
            - penalty::GAP_EXTENSION
    );

    assert_eq!(m.matched_ranges().sorted(), [3..6, 8..10]);
}

pub fn upstream_7<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "12356", "abc123 456");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        5 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 2) * bonus::CAMEL_123
            + bonus::CONSECUTIVE
            - penalty::GAP_START
            - penalty::GAP_EXTENSION
    );

    assert_eq!(m.matched_ranges().sorted(), [3..6, 8..10]);
}

pub use utils::*;

mod utils {
    use core::ops::Range;

    use norm::fzf::{FzfDistance, FzfParser, FzfQuery, FzfV1, FzfV2};
    use norm::{CaseSensitivity, Match, Metric};

    /// TODO: docs
    pub trait SortedRanges {
        fn sorted(&self) -> Vec<Range<usize>>;
    }

    impl SortedRanges for &[Range<usize>] {
        fn sorted(&self) -> Vec<Range<usize>> {
            let mut sorted = self.to_vec();
            sorted.sort_by_key(|r| r.start);
            sorted
        }
    }

    pub trait Fzf:
        Default
        + for<'a> Metric<Query<'a> = FzfQuery<'a>, Distance = FzfDistance>
    {
        fn with_case_sensitivity(
            self,
            case_sensitivity: CaseSensitivity,
        ) -> Self;

        fn with_matched_ranges(self, matched_ranges: bool) -> Self;

        fn scheme(&self) -> &norm::fzf::Scheme;
    }

    impl Fzf for FzfV1 {
        fn with_case_sensitivity(
            self,
            case_sensitivity: CaseSensitivity,
        ) -> Self {
            self.with_case_sensitivity(case_sensitivity)
        }

        fn with_matched_ranges(self, matched_ranges: bool) -> Self {
            self.with_matched_ranges(matched_ranges)
        }

        fn scheme(&self) -> &norm::fzf::Scheme {
            #[cfg(feature = "tests")]
            {
                self.scheme()
            }

            #[cfg(not(feature = "tests"))]
            {
                unreachable!()
            }
        }
    }

    impl Fzf for FzfV2 {
        fn with_case_sensitivity(
            self,
            case_sensitivity: CaseSensitivity,
        ) -> Self {
            self.with_case_sensitivity(case_sensitivity)
        }

        fn with_matched_ranges(self, matched_ranges: bool) -> Self {
            self.with_matched_ranges(matched_ranges)
        }

        fn scheme(&self) -> &norm::fzf::Scheme {
            #[cfg(feature = "tests")]
            {
                self.scheme()
            }

            #[cfg(not(feature = "tests"))]
            {
                unreachable!()
            }
        }
    }

    pub(super) fn fzf<F: Fzf>(
        case_sensitivity: CaseSensitivity,
        query: &str,
        candidate: &str,
    ) -> (F, Option<Match<FzfDistance>>) {
        let mut fzf = F::default()
            .with_case_sensitivity(case_sensitivity)
            .with_matched_ranges(true);

        let mut parser = FzfParser::new();

        let m = fzf.distance(parser.parse(query), candidate);

        (fzf, m)
    }
}
