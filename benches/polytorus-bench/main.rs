use polytorus::cryptography::falcon::falcon512;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;

fn bench_falcon512(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let msg: [u8; 5] = rng.gen();
    let (sk, pk) = falcon512::keygen(rng.gen());
    c.bench_function("falcon512", |b| b.iter(|| {
        let sig = falcon512::sign(&msg, &sk);
        falcon512::verify(&msg, &sig, &pk);
    }));
}

criterion_group!(benches, bench_falcon512);
criterion_main!(benches);
