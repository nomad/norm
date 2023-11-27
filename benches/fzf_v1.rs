use core::ops::Range;

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
    fzf::{FzfParser, FzfQuery, FzfV1},
    CaseSensitivity,
    Metric,
};

impl bench::Parser<FzfV1> for FzfParser {
    fn parse<'a>(&'a mut self, s: &str) -> FzfQuery<'a> {
        self.parse(s)
    }
}

impl bench::Metric for FzfV1 {
    type Query<'a> = FzfQuery<'a>;

    type Parser = FzfParser;

    #[inline]
    fn dist(&mut self, query: FzfQuery, candidate: &str) {
        self.distance(query, candidate);
    }
    #[inline(always)]
    fn dist_and_ranges(
        &mut self,
        query: FzfQuery,
        candidate: &str,
        ranges: &mut Vec<Range<usize>>,
    ) {
        self.distance_and_ranges(query, candidate, ranges);
    }
    fn set_case_sensitivity(
        &mut self,
        case_sensitivity: CaseSensitivity,
    ) -> &mut Self {
        self.set_case_sensitivity(case_sensitivity)
    }
}

fn group(c: &mut Criterion) -> BenchmarkGroup<WallTime> {
    c.benchmark_group("fzf_v1")
}

fn short(c: &mut Criterion) {
    bench::short(FzfV1::new(), None, group(c));
}

fn medium_start(c: &mut Criterion) {
    bench::medium_start(FzfV1::new(), None, group(c));
}

fn medium_middle(c: &mut Criterion) {
    bench::medium_middle(FzfV1::new(), None, group(c));
}

fn medium_end(c: &mut Criterion) {
    bench::medium_end(FzfV1::new(), None, group(c));
}

fn long_start(c: &mut Criterion) {
    bench::long_start(FzfV1::new(), None, group(c));
}

fn long_middle(c: &mut Criterion) {
    bench::long_middle(FzfV1::new(), None, group(c));
}

fn long_end(c: &mut Criterion) {
    bench::long_end(FzfV1::new(), None, group(c));
}

criterion_group!(
    benches,
    short,
    medium_start,
    medium_middle,
    medium_end,
    long_start,
    long_middle,
    long_end,
);
criterion_main!(benches);
