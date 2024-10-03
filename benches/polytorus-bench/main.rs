use polytorus::cryptography::falcon::{falcon512, falcon1024};
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

fn bench_falcon1024(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let msg: [u8; 5] = rng.gen();
    let (sk, pk) = falcon1024::keygen(rng.gen());
    c.bench_function("falcon1024", |b| b.iter(|| {
        let sig = falcon1024::sign(&msg, &sk);
        falcon1024::verify(&msg, &sig, &pk);
    }));
}

criterion_group!(benches, bench_falcon512, bench_falcon1024);
criterion_main!(benches);
