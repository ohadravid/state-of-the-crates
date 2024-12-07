use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::hint::black_box;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

use half::f16;
use ndarray::Array2;

fn f16_array(a: Array2<f16>, w: &Array2<f16>) -> Array2<f16> {
    a * w
}

fn f32_array(a: Array2<f32>, w: &Array2<f32>) -> Array2<f32> {
    a * w
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));

    c.bench_function("array f16", |b| {
        let a = Array2::from_shape_vec((128, 128), vec![f16::from_f32(0.1); 128 * 128]).unwrap();
        let w = Array2::from_shape_vec((128, 128), vec![f16::from_f32(0.2); 128 * 128]).unwrap();
        b.iter_batched(
            || (a.clone(), &w),
            |(a, w)| {
                let _ = f16_array(black_box(a), black_box(&w));
            },
            BatchSize::LargeInput,
        )
    });

    c.bench_function("array f32", |b| {
        let a = Array2::from_shape_vec((128, 128), vec![0.1; 128 * 128]).unwrap();
        let w = Array2::from_shape_vec((128, 128), vec![0.2; 128 * 128]).unwrap();
        b.iter_batched(
            || (a.clone(), &w),
            |(a, w)| {
                let _ = f32_array(black_box(a), black_box(&w));
            },
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
