use criterion::{criterion_group, criterion_main, Criterion};

use bluenoise::BlueNoise;

fn init_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("initialization");
    group.bench_function("10x10x1.0", |b| {
        b.iter(|| {
            let x = BlueNoise::new(10, 10, 1.0);
        })
    });

    group.bench_function("100x100x1.0", |b| {
        b.iter(|| {
            let x = BlueNoise::new(100, 100, 1.0);
        })
    });

    group.bench_function("1000x1000x1.0", |b| {
        b.iter(|| {
            let x = BlueNoise::new(1000, 1000, 1.0);
        })
    });
    group.finish();
}

fn execution_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("execution");

    let x = BlueNoise::new(10, 10, 1.0);
    group.bench_function("10x10x1.0", |b| b.iter(|| x.clone().count()));

    // generating roughly 7,200 points
    let x = BlueNoise::new(100, 100, 1.0);
    group.bench_function("100x100x1.0", |b| b.iter(|| x.clone().count()));

    // generating roughly 720,000 points
    let x = BlueNoise::new(1000, 1000, 1.0);
    group.bench_function("1000x1000x1.0", |b| b.iter(|| x.clone().count()));

    group.finish();
}

criterion_group!(benches, init_time, execution_time);
criterion_main!(benches);
