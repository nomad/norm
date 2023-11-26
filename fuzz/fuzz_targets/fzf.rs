#![no_main]

use libfuzzer_sys::arbitrary::{self, Arbitrary};
use libfuzzer_sys::fuzz_target;
use norm::fzf::{FzfParser, FzfScheme, FzfV1, FzfV2};
use norm::{CaseSensitivity, Metric};

#[derive(Arbitrary, Copy, Clone, Debug)]
struct Query<'a>(&'a str);

#[derive(Arbitrary, Clone, Debug)]
struct Candidate<'a>(&'a str);

fn with_opts<F>(mut fun: F)
where
    F: FnMut(CaseSensitivity, bool, FzfScheme),
{
    for case_sensitivity in [
        CaseSensitivity::Sensitive,
        CaseSensitivity::Insensitive,
        CaseSensitivity::Smart,
    ] {
        for normalization in [true, false] {
            for scheme in
                [FzfScheme::Default, FzfScheme::Path, FzfScheme::History]
            {
                fun(case_sensitivity, normalization, scheme)
            }
        }
    }
}

fuzz_target!(|data: (Query, Candidate)| {
    let (Query(query), Candidate(candidate)) = data;

    let mut parser = FzfParser::new();

    let query = parser.parse(query);

    let mut fzf_v1 = FzfV1::new();

    let mut fzf_v2 = FzfV2::new();

    let mut ranges = Vec::new();

    with_opts(|case_sensitivity, normalization, scheme| {
        let _ = fzf_v1
            .with_case_sensitivity(case_sensitivity)
            .with_normalization(normalization)
            .with_scoring_scheme(scheme)
            .distance_and_ranges(query, candidate, &mut ranges);

        for range in ranges.as_slice() {
            let _ = &candidate[range.clone()];
        }

        let _ = fzf_v2
            .with_case_sensitivity(case_sensitivity)
            .with_normalization(normalization)
            .with_scoring_scheme(scheme)
            .distance_and_ranges(query, candidate, &mut ranges);

        for range in ranges.as_slice() {
            let _ = &candidate[range.clone()];
        }
    });
});
