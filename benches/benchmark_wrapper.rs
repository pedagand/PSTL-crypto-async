extern crate crypto_async;
extern crate criterion;

use criterion::{criterion_group, criterion_main, Criterion};

fn wrapper_des_benchmark(c: &mut Criterion) {
    let mut plain: [u64; 64] = [0x0000000000000001; 64];
    for i in 0..64 {
        plain[i as usize] = 13 * i;
    }

    let mut key: [u64; 56] = [0x0000000000000; 56];
    for i in 0..56 {
        key[i as usize] = 7 * i;
    }

    let  cipher: [u64; 64] = [0x000000000000000; 64];

    c.bench_function("Bench the wrapper des function", 
                     move |b| b.iter(|| crypto_async::wrapper_des::des(plain, key, cipher)));
}

criterion_group!(benches, wrapper_des_benchmark);

criterion_main!(benches);
