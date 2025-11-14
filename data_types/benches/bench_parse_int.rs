use criterion::{Criterion, criterion_group, criterion_main};
use data_types::utils::parser_int::{
    parse_i8, parse_i16, parse_i16_unsafe, parse_i32, parse_i32_unsafe, parse_i64, parse_i64_unsafe,
};
use std::hint::black_box;

fn bench_parse_i8(c: &mut Criterion) {
    let bytes = [0x7F];
    c.bench_function("parse_i8 (safe)", |b| {
        b.iter(|| {
            let _ = black_box(parse_i8(black_box(&bytes))).unwrap();
        })
    });
}

fn bench_parse_i16(c: &mut Criterion) {
    let bytes = [0x12, 0x34];

    c.bench_function("parse_i16 (safe)", |b| {
        b.iter(|| {
            let _ = black_box(parse_i16(black_box(&bytes))).unwrap();
        })
    });

    c.bench_function("parse_i16 (unsafe)", |b| {
        b.iter(|| {
            let _ = unsafe { black_box(parse_i16_unsafe(black_box(&bytes))) };
        })
    });
}

fn bench_parse_i32(c: &mut Criterion) {
    let bytes = [0x00, 0x00, 0x00, 0x01];

    c.bench_function("parse_i32 (safe)", |b| {
        b.iter(|| {
            let _ = black_box(parse_i32(black_box(&bytes))).unwrap();
        })
    });

    c.bench_function("parse_i32 (unsafe)", |b| {
        b.iter(|| {
            let _ = unsafe { black_box(parse_i32_unsafe(black_box(&bytes))) };
        })
    });
}

fn bench_parse_i64(c: &mut Criterion) {
    let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];

    c.bench_function("parse_i64 (safe)", |b| {
        b.iter(|| {
            let _ = black_box(parse_i64(black_box(&bytes))).unwrap();
        })
    });

    c.bench_function("parse_i64 (unsafe)", |b| {
        b.iter(|| {
            let _ = unsafe { black_box(parse_i64_unsafe(black_box(&bytes))) };
        })
    });
}

//
// Register benchmarks
//
criterion_group!(
    benches,
    bench_parse_i8,
    bench_parse_i16,
    bench_parse_i32,
    bench_parse_i64
);
criterion_main!(benches);
