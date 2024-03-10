#![allow(dead_code)]

use core::ops::Range;

use criterion::{
    measurement::WallTime,
    BenchmarkGroup,
    BenchmarkId,
    Throughput,
};
use norm::CaseSensitivity;

pub trait Metric {
    type Query<'a>: Copy;

    type Parser: Parser<Self>;

    fn dist(&mut self, query: Self::Query<'_>, candidate: &str);

    fn dist_and_ranges(
        &mut self,
        query: Self::Query<'_>,
        candidate: &str,
        ranges: &mut Vec<Range<usize>>,
    );

    fn set_case_sensitivity(
        &mut self,
        case_sensitivity: CaseSensitivity,
    ) -> &mut Self;
}

pub trait Parser<M: Metric + ?Sized>: Default {
    fn parse<'a>(&'a mut self, s: &str) -> M::Query<'a>;
}

// TODO: docs
fn param(
    case: CaseSensitivity,
    with_ranges: bool,
    suffix: Option<&str>,
) -> String {
    let mut s = String::new();

    let case = match case {
        CaseSensitivity::Sensitive => "case_sensitive",
        CaseSensitivity::Insensitive => "case_insensitive",
        CaseSensitivity::Smart => "case_smart",
    };

    s.push_str(case);

    let ranges = if with_ranges { "_with_ranges" } else { "" };

    s.push_str(ranges);

    if let Some(suffix) = suffix {
        s.push('_');
        s.push_str(suffix);
    }

    s
}

// TODO: docs
fn for_all_cases_and_ranges<M, F>(
    mut metric: M,
    function: &str,
    suffix: Option<&str>,
    mut fun: F,
) where
    M: Metric,
    F: FnMut(&mut M, BenchmarkId, Option<&mut Vec<Range<usize>>>),
{
    for case in [
        CaseSensitivity::Sensitive,
        CaseSensitivity::Insensitive,
        CaseSensitivity::Smart,
    ] {
        for with_ranges in [true, false] {
            metric.set_case_sensitivity(case);
            let param = param(case, with_ranges, suffix);
            let mut ranges = with_ranges.then(Vec::new);
            fun(
                &mut metric,
                BenchmarkId::new(function, param),
                ranges.as_mut(),
            );
        }
    }
}

// TODO: docs
fn bench<'a, M, C>(
    group: &mut BenchmarkGroup<WallTime>,
    id: BenchmarkId,
    metric: &mut M,
    query: &str,
    candidates: C,
    ranges: Option<&mut Vec<Range<usize>>>,
) where
    M: Metric,
    C: IntoIterator<Item = &'a str>,
    C::IntoIter: ExactSizeIterator + Clone,
{
    let mut parser = M::Parser::default();

    let query = parser.parse(query);

    let candidates = candidates.into_iter();

    group.throughput(Throughput::Elements(candidates.len() as u64));

    if let Some(ranges) = ranges {
        group.bench_function(id, |b| {
            b.iter(|| {
                for candidate in candidates.clone() {
                    metric.dist_and_ranges(query, candidate, ranges);
                }
            })
        });
    } else {
        group.bench_function(id, |b| {
            b.iter(|| {
                for candidate in candidates.clone() {
                    metric.dist(query, candidate);
                }
            })
        });
    }
}

pub const MEDIUM_TEXT: &str =
    "Far far away, behind the word mountains, far from the countries Vokalia \
     and Consonantia, there live the blind texts. Separated they live in \
     Bookmarksgrove right at the coast of the Semantics, a large.";

pub const LONG_TEXT: &str =
    "Far far away, behind the word mountains, far from the countries Vokalia \
     and Consonantia, there live the blind texts. Separated they live in \
     Bookmarksgrove right at the coast of the Semantics, a large language \
     ocean. A small river named Duden flows by their place and supplies it \
     with the necessary regelialia. It is a paradisematic country, in which \
     roasted parts of sentences fly into your mouth. Even the all-powerful \
     Pointing has no control about the blind texts it is an almost \
     unorthographic life";

// TODO: docs
pub fn short<M: Metric>(
    metric: M,
    suffix: Option<&str>,
    mut group: BenchmarkGroup<WallTime>,
) {
    for_all_cases_and_ranges(metric, "short", suffix, |metric, id, ranges| {
        let query = "paradise";
        let candidates = core::iter::once("paradisematic");
        bench(&mut group, id, metric, query, candidates, ranges);
    })
}

// TODO: docs
pub fn medium_start<M: Metric>(
    metric: M,
    suffix: Option<&str>,
    mut group: BenchmarkGroup<WallTime>,
) {
    for_all_cases_and_ranges(
        metric,
        "medium_start",
        suffix,
        |metric, id, ranges| {
            let query = "away";
            let candidates = core::iter::once(MEDIUM_TEXT);
            bench(&mut group, id, metric, query, candidates, ranges);
        },
    )
}

// TODO: docs
pub fn medium_middle<M: Metric>(
    metric: M,
    suffix: Option<&str>,
    mut group: BenchmarkGroup<WallTime>,
) {
    for_all_cases_and_ranges(
        metric,
        "medium_middle",
        suffix,
        |metric, id, ranges| {
            let query = "blind";
            let candidates = core::iter::once(MEDIUM_TEXT);
            bench(&mut group, id, metric, query, candidates, ranges);
        },
    )
}

// TODO: docs
pub fn medium_end<M: Metric>(
    metric: M,
    suffix: Option<&str>,
    mut group: BenchmarkGroup<WallTime>,
) {
    for_all_cases_and_ranges(
        metric,
        "medium_end",
        suffix,
        |metric, id, ranges| {
            let query = "Semantics";
            let candidates = core::iter::once(MEDIUM_TEXT);
            bench(&mut group, id, metric, query, candidates, ranges);
        },
    )
}

// TODO: docs
pub fn long_start<M: Metric>(
    metric: M,
    suffix: Option<&str>,
    mut group: BenchmarkGroup<WallTime>,
) {
    for_all_cases_and_ranges(
        metric,
        "long_start",
        suffix,
        |metric, id, ranges| {
            let query = "mountains";
            let candidates = core::iter::once(LONG_TEXT);
            bench(&mut group, id, metric, query, candidates, ranges);
        },
    )
}

// TODO: docs
pub fn long_middle<M: Metric>(
    metric: M,
    suffix: Option<&str>,
    mut group: BenchmarkGroup<WallTime>,
) {
    for_all_cases_and_ranges(
        metric,
        "long_middle",
        suffix,
        |metric, id, ranges| {
            let query = "Duden";
            let candidates = core::iter::once(LONG_TEXT);
            bench(&mut group, id, metric, query, candidates, ranges);
        },
    )
}

// TODO: docs
pub fn long_end<M: Metric>(
    metric: M,
    suffix: Option<&str>,
    mut group: BenchmarkGroup<WallTime>,
) {
    for_all_cases_and_ranges(
        metric,
        "long_end",
        suffix,
        |metric, id, ranges| {
            let query = "unorthographic";
            let candidates = core::iter::once(LONG_TEXT);
            bench(&mut group, id, metric, query, candidates, ranges);
        },
    )
}
