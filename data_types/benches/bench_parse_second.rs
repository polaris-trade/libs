use criterion::{Criterion, criterion_group, criterion_main};
use data_types::time::second::UnixSeconds;
use std::hint::black_box;

fn bench_seconds_from_bytes_safe(c: &mut Criterion) {
    let bytes = [0x00, 0x00, 0x00, 0x00, 0x3B, 0x9A, 0xCA, 0x00];
    c.bench_function("Seconds::from_bytes (safe)", |b| {
        b.iter(|| {
            let result = UnixSeconds::from_bytes(black_box(&bytes));
            black_box(result)
        })
    });
}

fn bench_seconds_from_bytes_u32_safe(c: &mut Criterion) {
    let bytes = [0x3B, 0x9A, 0xCA, 0x00];
    c.bench_function("Seconds::from_bytes_u32 (safe)", |b| {
        b.iter(|| {
            let result = UnixSeconds::from_bytes_u32(black_box(&bytes));
            black_box(result)
        })
    });
}

fn bench_seconds_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("Seconds batch processing");

    let base_timestamp = 1577836800u64; // Jan 1, 2020
    let test_data: Vec<[u8; 8]> = (0..1000)
        .map(|i| (base_timestamp + (i as u64 * 86400)).to_be_bytes())
        .collect();

    group.bench_function("batch safe", |b| {
        b.iter(|| {
            let results: Vec<_> = test_data
                .iter()
                .map(|bytes| UnixSeconds::from_bytes(black_box(bytes)))
                .collect();
            black_box(results)
        })
    });

    group.finish();
}

fn bench_seconds_datetime_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("Seconds datetime conversion");

    let timestamp_bytes = [0x00, 0x00, 0x00, 0x00, 0x5E, 0x0B, 0xE1, 0x00];

    group.bench_function("parse and convert to UTC (safe)", |b| {
        b.iter(|| {
            let seconds = UnixSeconds::from_bytes(black_box(&timestamp_bytes)).unwrap();
            let dt = seconds.to_utc();
            black_box(dt)
        })
    });

    group.bench_function("parse and convert to local (safe)", |b| {
        b.iter(|| {
            let seconds = UnixSeconds::from_bytes(black_box(&timestamp_bytes)).unwrap();
            let dt = seconds.to_local();
            black_box(dt)
        })
    });

    group.finish();
}

fn bench_seconds_type_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("Seconds type conversion");

    let timestamp_bytes = [0x00, 0x00, 0x00, 0x00, 0x5E, 0x0B, 0xE1, 0x00];

    group.bench_function("seconds to nanoseconds (safe)", |b| {
        b.iter(|| {
            let seconds = UnixSeconds::from_bytes(black_box(&timestamp_bytes)).unwrap();
            let nanoseconds = seconds.to_nanoseconds().unwrap();
            black_box(nanoseconds)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_seconds_from_bytes_safe,
    bench_seconds_from_bytes_u32_safe,
    bench_seconds_batch_processing,
    bench_seconds_datetime_conversion,
    bench_seconds_type_conversion
);
criterion_main!(benches);
