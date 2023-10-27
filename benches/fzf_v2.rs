mod common;

use common as bench;
use criterion::{
    criterion_group,
    criterion_main,
    measurement::WallTime,
    BenchmarkGroup,
    Criterion,
};
use norm::{
    fzf::{FzfQuery, FzfV2},
    CaseSensitivity,
    Metric,
};

impl<'a> bench::FromStr<'a> for FzfQuery<'a> {
    fn from_str(s: &'a str) -> Self {
        FzfQuery::new(s)
    }
}

impl bench::Metric for FzfV2 {
    type Query<'a> = FzfQuery<'a>;

    #[inline]
    fn dist(&mut self, query: FzfQuery, candidate: &str) {
        self.distance(query, candidate);
    }
    fn with_case_sensitivity(self, case_sensitivity: CaseSensitivity) -> Self {
        self.with_case_sensitivity(case_sensitivity)
    }
    fn with_matched_ranges(self, matched_ranges: bool) -> Self {
        self.with_matched_ranges(matched_ranges)
    }
}

fn group(c: &mut Criterion) -> BenchmarkGroup<WallTime> {
    c.benchmark_group("fzf_v1")
}

fn short(c: &mut Criterion) {
    bench::short(FzfV2::new(), None, group(c));
}

criterion_group!(benches, short);
criterion_main!(benches);
