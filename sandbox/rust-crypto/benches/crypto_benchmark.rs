#[macro_use]
extern crate criterion;
extern crate crypto;
extern crate rustc_serialize;

use crypto::symmetriccipher::{BlockEncryptor, BlockEncryptorX8};
use crypto::aessafe::{AesSafe128Encryptor, AesSafe128EncryptorX8};
use criterion::{Criterion,Throughput, Benchmark };
use crypto::aes::{self, KeySize};



fn encrypt(input : &[u8], key : &[u8]) -> [u8;16] {
    let mut output =  [0u8;16];
    //initialize the Encryptor
    let encryptor = AesSafe128Encryptor::new(&key);
    encryptor.encrypt_block(&input, &mut output);
    return output
}

fn encrypt_x8(input : &[u8], key : &[u8]) -> [u8;128] {
    let mut output =  [0u8;128];
    //initialize the Encryptor
    let encryptor = AesSafe128EncryptorX8::new(&key);
    encryptor.encrypt_block_x8(&input, &mut output);
    return output
}

fn ctr_encryption(input : &[u8], output : &mut [u8], key : &[u8], nonce : &[u8]) {
    //initialize the Encryptor
    let mut _cipher = aes::ctr(KeySize::KeySize128, key, nonce);
    _cipher.process(input, &mut output[..]);
}

fn encrypt_benchmark(c: &mut Criterion) {


    static INPUT_X8 : &[u8;128] = &[0;128];
    static INPUT : &[u8;16] = &[0;16];
    static KEY: &[u8;16] = &[0;16];
    static NONCE : &[u8; 16] = &[1;16];
    let mut output : [u8;16] =[0;16];

    //c.bench_function("Classic Encrypt", move |b| b.iter(|| encrypt(&input, &key)));
    //c.bench_function("x8 Encrypt", move |b| b.iter(|| encrypt_x8(&input_x8, &key)));
    c.bench(
        "throughput Encrypt",
        Benchmark::new(
            "classic Encrypt",
            |b| b.iter(|| encrypt(INPUT, KEY)),
        ).throughput(Throughput::Bytes(INPUT.len() as u32)),
    );

    c.bench(
        "throughput Encrypt x8",
        Benchmark::new(
            "x8 Encrypt",
            |b| b.iter(|| encrypt_x8(INPUT_X8, KEY)),
        ).throughput(Throughput::Bytes(INPUT_X8.len() as u32)),
    );

    c.bench(
        "throughput Ctr encryption",
        Benchmark::new(
            "Ctr encryption",
            move |b| b.iter(|| ctr_encryption(INPUT, &mut output, KEY, NONCE)),
        ).throughput(Throughput::Bytes(INPUT.len() as u32)),
    );

}

criterion_group!(benches, encrypt_benchmark);
criterion_main!(benches);
