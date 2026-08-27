use claims::assert_ok;
use criterion::{Criterion, criterion_group, criterion_main};
use kristofersxyz::authentication::{Password, compute_password_hash};
use std::{hint::black_box, thread, time::Duration};

fn password_hashing(criterion: &mut Criterion) {
    let password = assert_ok!(Password::try_from("benchmark password input".to_owned()));
    let mut group = criterion.benchmark_group("argon2id");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(15));

    group.bench_function("one_hash", |bencher| {
        bencher.iter(|| black_box(compute_password_hash(black_box(&password))));
    });
    group.bench_function("two_concurrent_hashes", |bencher| {
        bencher.iter(|| {
            thread::scope(|scope| {
                let first = scope.spawn(|| compute_password_hash(black_box(&password)));
                let second = scope.spawn(|| compute_password_hash(black_box(&password)));
                black_box((first.join(), second.join()))
            })
        });
    });
    group.finish();
}

criterion_group!(benches, password_hashing);
criterion_main!(benches);
