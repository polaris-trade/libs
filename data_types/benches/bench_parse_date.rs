use criterion::{Criterion, criterion_group, criterion_main};
use data_types::time::date::Date;
use std::{convert::TryFrom, hint::black_box};

const SAMPLE_BYTES: [u8; 4] = 20251024u32.to_be_bytes();

fn bench_safe_parse(c: &mut Criterion) {
    c.bench_function("Date::try_from_safe", |b| {
        b.iter(|| {
            let date = Date::try_from(black_box(&SAMPLE_BYTES[..])).unwrap();
            black_box(date);
        })
    });
}

fn bench_unsafe_parse(c: &mut Criterion) {
    c.bench_function("Date::from_bytes_unchecked", |b| {
        b.iter(|| {
            let date = unsafe { Date::from_bytes_unchecked(black_box(&SAMPLE_BYTES[..])) };
            black_box(date);
        })
    });
}

fn bench_to_naive_date(c: &mut Criterion) {
    let date = Date::try_from(&SAMPLE_BYTES[..]).unwrap();
    c.bench_function("Date::to_naive_date", |b| {
        b.iter(|| {
            let nd = date.to_naive_date();
            black_box(nd);
        })
    });
}

criterion_group!(
    date_benches,
    bench_safe_parse,
    bench_unsafe_parse,
    bench_to_naive_date
);
criterion_main!(date_benches);
