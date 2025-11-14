use criterion::{Criterion, criterion_group, criterion_main};
use data_types::time::nanosecond::UnixNanoseconds;
use std::hint::black_box;

fn bench_nanoseconds_from_bytes_safe(c: &mut Criterion) {
    let bytes = [0x11, 0x2E, 0x16, 0x5C, 0x8F, 0x7A, 0x4E, 0x00];
    c.bench_function("Nanoseconds::from_bytes (safe)", |b| {
        b.iter(|| {
            let result = UnixNanoseconds::from_bytes(black_box(&bytes));
            black_box(result)
        })
    });
}

fn bench_nanoseconds_from_bytes_u32_safe(c: &mut Criterion) {
    let bytes = [0x3B, 0x9A, 0xCA, 0x00];
    c.bench_function("Nanoseconds::from_bytes_u32 (safe)", |b| {
        b.iter(|| {
            let result = UnixNanoseconds::from_bytes_u32(black_box(&bytes));
            black_box(result)
        })
    });
}

fn bench_nanoseconds_conversion_safe(c: &mut Criterion) {
    let mut group = c.benchmark_group("Nanoseconds conversion comparison");

    let bytes_u64 = [0x11, 0x2E, 0x16, 0x5C, 0x8F, 0x7A, 0x4E, 0x00];
    let bytes_u32 = [0x3B, 0x9A, 0xCA, 0x00];

    group.bench_function("u64 safe", |b| {
        b.iter(|| {
            let result = UnixNanoseconds::from_bytes(black_box(&bytes_u64));
            black_box(result)
        })
    });

    group.bench_function("u32 safe", |b| {
        b.iter(|| {
            let result = UnixNanoseconds::from_bytes_u32(black_box(&bytes_u32));
            black_box(result)
        })
    });

    group.finish();
}

fn bench_nanoseconds_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("Nanoseconds batch processing");

    let test_data: Vec<[u8; 8]> = (0..1000)
        .map(|i| ((i as u64 * 1_000_000_000) + i as u64).to_be_bytes())
        .collect();

    group.bench_function("batch safe", |b| {
        b.iter(|| {
            let results: Vec<_> = test_data
                .iter()
                .map(|bytes| UnixNanoseconds::from_bytes(black_box(bytes)))
                .collect();
            black_box(results)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_nanoseconds_from_bytes_safe,
    bench_nanoseconds_from_bytes_u32_safe,
    bench_nanoseconds_conversion_safe,
    bench_nanoseconds_batch_processing
);
criterion_main!(benches);
