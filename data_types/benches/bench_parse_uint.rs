use criterion::{Criterion, criterion_group, criterion_main};
use data_types::utils::parser_uint::{
    parse_u8, parse_u16, parse_u16_unsafe, parse_u32, parse_u32_unsafe, parse_u64, parse_u64_unsafe,
};
use std::hint::black_box;

fn bench_parse_u8(c: &mut Criterion) {
    let bytes = [0x7F];
    c.bench_function("parse_u8 (safe)", |b| {
        b.iter(|| {
            let _ = black_box(parse_u8(black_box(&bytes))).unwrap();
        })
    });
}

fn bench_parse_u16(c: &mut Criterion) {
    let bytes = [0x12, 0x34];

    c.bench_function("parse_u16 (safe)", |b| {
        b.iter(|| {
            let _ = black_box(parse_u16(black_box(&bytes))).unwrap();
        })
    });

    c.bench_function("parse_u16 (unsafe)", |b| {
        b.iter(|| {
            let _ = unsafe { black_box(parse_u16_unsafe(black_box(&bytes))) };
        })
    });
}

fn bench_parse_u32(c: &mut Criterion) {
    let bytes = [0x00, 0x00, 0x00, 0x01];

    c.bench_function("parse_u32 (safe)", |b| {
        b.iter(|| {
            let _ = black_box(parse_u32(black_box(&bytes))).unwrap();
        })
    });

    c.bench_function("parse_u32 (unsafe)", |b| {
        b.iter(|| {
            let _ = unsafe { black_box(parse_u32_unsafe(black_box(&bytes))) };
        })
    });
}

fn bench_parse_u64(c: &mut Criterion) {
    let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];

    c.bench_function("parse_u64 (safe)", |b| {
        b.iter(|| {
            let _ = black_box(parse_u64(black_box(&bytes))).unwrap();
        })
    });

    c.bench_function("parse_u64 (unsafe)", |b| {
        b.iter(|| {
            let _ = unsafe { black_box(parse_u64_unsafe(black_box(&bytes))) };
        })
    });
}

// Register benchmarks
criterion_group!(
    benches_uint,
    bench_parse_u8,
    bench_parse_u16,
    bench_parse_u32,
    bench_parse_u64
);
criterion_main!(benches_uint);
