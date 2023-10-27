#![allow(clippy::single_range_in_vec_init)]

use norm::fzf::{bonus, penalty};
use norm::CaseSensitivity;
use CaseSensitivity::*;

pub fn empty_query<F: Fzf>() {
    assert!(fzf::<F>(Insensitive, "", "foo").is_none());
}

pub fn upstream_1<F: Fzf>() {
    let m = fzf::<F>(Insensitive, "oBZ", "fooBarbaz").unwrap();

    assert_eq!(
        m.distance().into_score(),
        bonus::MATCH * 3 + bonus::CAMEL_123
            - penalty::GAP_START
            - penalty::GAP_EXTENSION * 3
    );

    assert_eq!(m.matched_ranges().sorted(), [2..4, 8..9]);
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
    }

    pub(super) fn fzf<F: Fzf>(
        case_sensitivity: CaseSensitivity,
        query: &str,
        candidate: &str,
    ) -> Option<Match<FzfDistance>> {
        let mut fzf = F::default()
            .with_case_sensitivity(case_sensitivity)
            .with_matched_ranges(true);

        let mut parser = FzfParser::new();

        fzf.distance(parser.parse(query), candidate)
    }
}
