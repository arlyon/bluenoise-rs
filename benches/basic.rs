use criterion::{criterion_group, criterion_main, Criterion};

use bluenoise::{BlueNoise, WrappingBlueNoise};
use rand_pcg::Pcg64Mcg;

fn init_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("initialization");
    group.bench_function("10x10x1.0", |b| {
        b.iter(|| {
            let _x = BlueNoise::<Pcg64Mcg>::new(10.0, 10.0, 1.0);
        })
    });

    group.bench_function("100x100x1.0", |b| {
        b.iter(|| {
            let _x = BlueNoise::<Pcg64Mcg>::new(100.0, 100.0, 1.0);
        })
    });

    group.bench_function("1000x1000x1.0", |b| {
        b.iter(|| {
            let _x = BlueNoise::<Pcg64Mcg>::new(1000.0, 1000.0, 1.0);
        })
    });

    // No need to benchmark the initialization of WrappingBlueNoise, since it's
    // just initialized as a newtype around BlueNoise.

    group.finish();
}

fn execution_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("execution");

    // generating roughly 80 points
    let x = BlueNoise::<Pcg64Mcg>::new(10.0, 10.0, 1.0);
    group.bench_function("10x10x1.0", |b| b.iter(|| x.clone().count()));

    // generating roughly 7,500 points
    let x = BlueNoise::<Pcg64Mcg>::new(100.0, 100.0, 1.0);
    group.bench_function("100x100x1.0", |b| b.iter(|| x.clone().count()));

    // generating roughly 750,000 points
    let x = BlueNoise::<Pcg64Mcg>::new(1000.0, 1000.0, 1.0);
    group.bench_function("1000x1000x1.0", |b| b.iter(|| x.clone().count()));

    // generating roughly 80 points
    let x = WrappingBlueNoise::<Pcg64Mcg>::new(10.0, 10.0, 1.0);
    group.bench_function("wrapping 10x10x1.0", |b| b.iter(|| x.clone().count()));

    // generating roughly 7,500 points
    let x = WrappingBlueNoise::<Pcg64Mcg>::new(100.0, 100.0, 1.0);
    group.bench_function("wrapping 100x100x1.0", |b| b.iter(|| x.clone().count()));

    // generating roughly 750,000 points
    let x = WrappingBlueNoise::<Pcg64Mcg>::new(1000.0, 1000.0, 1.0);
    group.bench_function("wrapping 1000x1000x1.0", |b| b.iter(|| x.clone().count()));

    group.finish();
}

criterion_group!(benches, init_time, execution_time);
criterion_main!(benches);
