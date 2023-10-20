use criterion::{
    criterion_group,
    criterion_main,
    measurement::WallTime,
    Bencher,
    BenchmarkGroup,
    Criterion,
};
use norm::{
    fzf::{FzfQuery, FzfV1},
    CaseSensitivity,
    Metric,
};

fn short(fzf: &FzfV1, b: &mut Bencher) {
    let jelly = FzfQuery::new("jelly");
    b.iter(|| fzf.distance(jelly, "jellyfish"))
}

fn sensitive_with_ranges() -> FzfV1 {
    FzfV1::new()
        .with_case_sensitivity(CaseSensitivity::Sensitive)
        .with_matched_ranges(true)
}

fn short_case_sensitive_with_ranges(c: &mut Criterion) {
    let fzf = sensitive_with_ranges();

    group(c).bench_function("short_case_sensitive_with_ranges", |b| {
        short(&fzf, b)
    });
}

fn group(c: &mut Criterion) -> BenchmarkGroup<WallTime> {
    c.benchmark_group("fzf_v1")
}

criterion_group!(benches, short_case_sensitive_with_ranges);
criterion_main!(benches);
