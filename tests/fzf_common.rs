#![allow(clippy::single_range_in_vec_init)]

use norm::fzf::{bonus, penalty};
use norm::CaseSensitivity;
use CaseSensitivity::*;

pub fn upstream_empty<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "", "foo");

    let m = m.unwrap();

    assert_eq!(m.distance().into_score(), 0);

    assert!(m.matched_ranges().is_empty());
}

pub fn upstream_fuzzy_1<F: Fzf>() {
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

pub fn upstream_fuzzy_2<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "fbb", "foo bar baz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 2)
                * fzf.scheme().bonus_boundary_white
            - 2 * penalty::GAP_START
            - 4 * penalty::GAP_EXTENSION
    );

    assert_eq!(m.matched_ranges().sorted(), [0..1, 4..5, 8..9]);
}

pub fn upstream_fuzzy_3<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "rdoc", "/AutomatorDocument.icns");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        4 * bonus::MATCH + 2 * bonus::CONSECUTIVE + bonus::CAMEL_123
    );

    assert_eq!(m.matched_ranges().sorted(), [9..13]);
}

pub fn upstream_fuzzy_4<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "zshc", "/man1/zshcompctl.1");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        4 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 3)
                * fzf.scheme().bonus_boundary_delimiter
    );

    assert_eq!(m.matched_ranges().sorted(), [6..10]);
}

pub fn upstream_fuzzy_5<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "zshc", "/.oh-my-zsh/cache");

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

pub fn upstream_fuzzy_6<F: Fzf>() {
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

pub fn upstream_fuzzy_7<F: Fzf>() {
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

pub fn upstream_fuzzy_8<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "fbb", "foo/bar/baz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH
            + bonus::FIRST_QUERY_CHAR_MULTIPLIER
                * fzf.scheme().bonus_boundary_white
            + 2 * fzf.scheme().bonus_boundary_delimiter
            - 2 * penalty::GAP_START
            - 4 * penalty::GAP_EXTENSION
    );

    assert_eq!(m.matched_ranges().sorted(), [0..1, 4..5, 8..9]);
}

pub fn upstream_fuzzy_9<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "fbb", "fooBarBaz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH
            + bonus::FIRST_QUERY_CHAR_MULTIPLIER
                * fzf.scheme().bonus_boundary_white
            + 2 * bonus::CAMEL_123
            - 2 * penalty::GAP_START
            - 2 * penalty::GAP_EXTENSION
    );

    assert_eq!(m.matched_ranges().sorted(), [0..1, 3..4, 6..7]);
}

pub fn upstream_fuzzy_10<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "fbb", "foo barbaz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 1)
                * fzf.scheme().bonus_boundary_white
            - 2 * penalty::GAP_START
            - 3 * penalty::GAP_EXTENSION
    );

    assert_eq!(m.matched_ranges().sorted(), [0..1, 4..5, 7..8]);
}

pub fn upstream_fuzzy_11<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "foob", "fooBar Baz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        4 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 3)
                * fzf.scheme().bonus_boundary_white
    );

    assert_eq!(m.matched_ranges().sorted(), [0..4]);
}

pub fn upstream_fuzzy_12<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "foo-b", "xFoo-Bar Baz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        5 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 2) * bonus::CAMEL_123
            + bonus::NON_WORD
            + bonus::BOUNDARY
    );

    assert_eq!(m.matched_ranges().sorted(), [1..6]);
}

pub fn upstream_fuzzy_13<F: Fzf>() {
    let (_, m) = fzf::<F>(Sensitive, "oBz", "fooBarbaz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH + bonus::CAMEL_123
            - penalty::GAP_START
            - 3 * penalty::GAP_EXTENSION
    );

    assert_eq!(m.matched_ranges().sorted(), [2..4, 8..9]);
}

pub fn upstream_fuzzy_14<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Sensitive, "FBB", "Foo/Bar/Baz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH
            + bonus::FIRST_QUERY_CHAR_MULTIPLIER
                * fzf.scheme().bonus_boundary_white
            + 2 * fzf.scheme().bonus_boundary_delimiter
            - 2 * penalty::GAP_START
            - 4 * penalty::GAP_EXTENSION
    );

    assert_eq!(m.matched_ranges().sorted(), [0..1, 4..5, 8..9]);
}

pub fn upstream_fuzzy_15<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Sensitive, "FBB", "FooBarBaz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH
            + bonus::FIRST_QUERY_CHAR_MULTIPLIER
                * fzf.scheme().bonus_boundary_white
            + 2 * bonus::CAMEL_123
            - 2 * penalty::GAP_START
            - 2 * penalty::GAP_EXTENSION
    );

    assert_eq!(m.matched_ranges().sorted(), [0..1, 3..4, 6..7]);
}

pub fn upstream_fuzzy_16<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Sensitive, "FooB", "FooBar Baz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        4 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 2)
                * fzf.scheme().bonus_boundary_white
            + bonus::CAMEL_123.max(fzf.scheme().bonus_boundary_white)
    );

    assert_eq!(m.matched_ranges().sorted(), [0..4]);
}

pub fn upstream_fuzzy_17<F: Fzf>() {
    let (_, m) = fzf::<F>(Sensitive, "o-ba", "foo-bar");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        4 * bonus::MATCH + 3 * bonus::BOUNDARY
    );

    assert_eq!(m.matched_ranges().sorted(), [2..6]);
}

pub fn upstream_fuzzy_18<F: Fzf>() {
    let (_, m) = fzf::<F>(Sensitive, "oBZ", "fooBarbaz");
    assert!(m.is_none());
}

pub fn upstream_fuzzy_19<F: Fzf>() {
    let (_, m) = fzf::<F>(Sensitive, "fbb", "Foo Bar Baz");
    assert!(m.is_none());
}

pub fn upstream_fuzzy_20<F: Fzf>() {
    let (_, m) = fzf::<F>(Sensitive, "fooBarbazz", "fooBarbaz");
    assert!(m.is_none());
}

pub fn upstream_exact_1<F: Fzf>() {
    let (_, m) = fzf::<F>(Sensitive, "'oBA", "fooBarbaz");
    assert!(m.is_none());
}

pub fn upstream_exact_2<F: Fzf>() {
    let (_, m) = fzf::<F>(Sensitive, "'fooBarbazz", "fooBarbaz");
    assert!(m.is_none());
}

pub fn upstream_exact_3<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "'oBA", "fooBarbaz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH + bonus::CAMEL_123 + bonus::CONSECUTIVE
    );

    assert_eq!(m.matched_ranges().sorted(), [2..5]);
}

pub fn upstream_exact_4<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "'rdoc", "/AutomatorDocument.icns");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        4 * bonus::MATCH + bonus::CAMEL_123 + 2 * bonus::CONSECUTIVE
    );

    assert_eq!(m.matched_ranges().sorted(), [9..13]);
}

pub fn upstream_exact_5<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "'zshc", "/man1/zshcompctl.1");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        4 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 3)
                * fzf.scheme().bonus_boundary_delimiter
    );

    assert_eq!(m.matched_ranges().sorted(), [6..10]);
}

pub fn upstream_exact_6<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "'zsh/c", "/.oh-my-zsh/cache");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        5 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 3) * bonus::BOUNDARY
            + fzf.scheme().bonus_boundary_delimiter
    );

    assert_eq!(m.matched_ranges().sorted(), [8..13]);
}

pub fn upstream_exact_7<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "'oo", "foobar foo");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        2 * bonus::MATCH + bonus::CONSECUTIVE
    );

    assert_eq!(m.matched_ranges().sorted(), [1..3]);
}

pub fn upstream_prefix_1<F: Fzf>() {
    let (_, m) = fzf::<F>(Sensitive, "^Foo", "fooBarbaz");
    assert!(m.is_none());
}

pub fn upstream_prefix_2<F: Fzf>() {
    let (_, m) = fzf::<F>(Sensitive, "^baz", "fooBarBaz");
    assert!(m.is_none());
}

pub fn upstream_prefix_3<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "^Foo", "fooBarbaz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 2)
                * fzf.scheme().bonus_boundary_white
    );

    assert_eq!(m.matched_ranges().sorted(), [0..3]);
}

pub fn upstream_prefix_4<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "^foo", "foOBarBaZ");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 2)
                * fzf.scheme().bonus_boundary_white
    );

    assert_eq!(m.matched_ranges().sorted(), [0..3]);
}

pub fn upstream_prefix_5<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "^f-o", "f-oBarbaz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 2)
                * fzf.scheme().bonus_boundary_white
    );

    assert_eq!(m.matched_ranges().sorted(), [0..3]);
}

pub fn upstream_prefix_6<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "^foo", " fooBar");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 2)
                * fzf.scheme().bonus_boundary_white
    );

    assert_eq!(m.matched_ranges().sorted(), [1..4]);
}

pub fn upstream_prefix_7<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "\\ fo", " fooBar");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 2)
                * fzf.scheme().bonus_boundary_white
    );

    assert_eq!(m.matched_ranges().sorted(), [0..3]);
}

pub fn upstream_prefix_8<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "^foo", "     fo");
    assert!(m.is_none());
}

pub fn upstream_suffix_1<F: Fzf>() {
    let (_, m) = fzf::<F>(Sensitive, "Baz$", "fooBarbaz");
    assert!(m.is_none());
}

pub fn upstream_suffix_2<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "Foo$", "fooBarBaz");
    assert!(m.is_none());
}

pub fn upstream_suffix_3<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "baz$", "fooBarbaz");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH + 2 * bonus::CONSECUTIVE
    );

    assert_eq!(m.matched_ranges().sorted(), [6..9]);
}

pub fn upstream_suffix_4<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "baz$", "fooBarBaZ");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH
            + (bonus::FIRST_QUERY_CHAR_MULTIPLIER + 2) * bonus::CAMEL_123
    );

    assert_eq!(m.matched_ranges().sorted(), [6..9]);
}

pub fn upstream_suffix_5<F: Fzf>() {
    let (_, m) = fzf::<F>(Insensitive, "baz$", "fooBarbaz ");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        3 * bonus::MATCH + 2 * bonus::CONSECUTIVE
    );

    assert_eq!(m.matched_ranges().sorted(), [6..9]);
}

pub fn upstream_suffix_6<F: Fzf>() {
    let (fzf, m) = fzf::<F>(Insensitive, "baz\\ $", "fooBarbaz ");

    let m = m.unwrap();

    assert_eq!(
        m.distance().into_score(),
        4 * bonus::MATCH
            + 2 * bonus::CONSECUTIVE
            + fzf.scheme().bonus_boundary_white
    );

    assert_eq!(m.matched_ranges().sorted(), [6..10]);
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
