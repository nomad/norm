use criterion::{
    measurement::WallTime,
    BenchmarkGroup,
    BenchmarkId,
    Throughput,
};
use norm::CaseSensitivity;

pub trait Metric {
    type Query<'a>: Copy + FromStr<'a>;

    fn dist(&self, query: Self::Query<'_>, candidate: &str);
    fn with_case_sensitivity(self, case_sensitivity: CaseSensitivity) -> Self;
    fn with_matched_ranges(self, matched_ranges: bool) -> Self;
}

pub trait FromStr<'a> {
    fn from_str(s: &'a str) -> Self;
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
    F: FnMut(&M, BenchmarkId),
{
    for case in [
        CaseSensitivity::Sensitive,
        CaseSensitivity::Insensitive,
        CaseSensitivity::Smart,
    ] {
        for with_ranges in [true, false] {
            metric = metric
                .with_case_sensitivity(case)
                .with_matched_ranges(with_ranges);

            let param = param(case, with_ranges, suffix);

            fun(&metric, BenchmarkId::new(function, param));
        }
    }
}

// TODO: docs
fn bench<'a, M, C>(
    group: &mut BenchmarkGroup<WallTime>,
    id: BenchmarkId,
    metric: &M,
    query: &str,
    candidates: C,
) where
    M: Metric,
    C: IntoIterator<Item = &'a str>,
    C::IntoIter: ExactSizeIterator + Clone,
{
    let query = M::Query::from_str(query);

    let candidates = candidates.into_iter();

    group.throughput(Throughput::Elements(candidates.len() as u64));

    group.bench_function(id, |b| {
        b.iter(|| {
            for candidate in candidates.clone() {
                metric.dist(query, candidate);
            }
        })
    });
}

// TODO: docs
pub fn short<M: Metric>(
    metric: M,
    suffix: Option<&str>,
    mut group: BenchmarkGroup<WallTime>,
) where
    M: Metric,
{
    for_all_cases_and_ranges(metric, "short", suffix, |metric, id| {
        let query = "jelly";
        let candidates = core::iter::once("jellyfish");
        bench(&mut group, id, metric, query, candidates);
    })
}
