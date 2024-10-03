use polytorus::cryptography::falcon::{falcon512, falcon1024};
use polytorus::wallet::wallets::Wallet;
use polytorus::wallet::transaction::Transaction;
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

fn bench_transaction(c: &mut Criterion) {
    let wallet = Wallet::new();
    let transaction = Transaction::new(wallet.clone(), "recipient".to_string(), 10).unwrap();
    c.bench_function("transaction", |b| b.iter(|| {
        let signed_transaction = transaction.sign(&wallet);
        signed_transaction.verify();
    }));
}

fn bench_transaction_update(c: &mut Criterion) {
    let wallet = Wallet::new();
    let mut transaction = Transaction::new(wallet.clone(), "recipient".to_string(), 10).unwrap();
    c.bench_function("transaction_update", |b| b.iter(|| {
        let _ = transaction.update(wallet.clone(), "recipient".to_string(), 5).unwrap();
    }));
}

criterion_group!(benches, bench_falcon512, bench_falcon1024, bench_transaction, bench_transaction_update);
criterion_main!(benches);
