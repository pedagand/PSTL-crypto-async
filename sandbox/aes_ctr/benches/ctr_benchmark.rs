#[macro_use]
extern crate criterion;
extern crate crypto;
extern crate rustc_serialize;

use crate::crypto::symmetriccipher::BlockEncryptor;
use crate::crypto::symmetriccipher::BlockEncryptorX8;
use crate::crypto::buffer::WriteBuffer;
use crate::crypto::buffer::ReadBuffer;
use crypto::aes::{self, KeySize};
use crypto::symmetriccipher::SynchronousStreamCipher;
use crypto::aessafe::AesSafe128Encryptor;
use std::iter::repeat;
use crypto::blockmodes::CtrModeX8;
use crypto::aessafe;
use crypto::buffer::OwnedReadBuffer;
use std::{cmp, ptr};
use criterion::{Criterion,Throughput, Benchmark };




fn ctr_benchmark(c: &mut Criterion) {


    static CTR_INPUT : &[u8;20] = &[0;20];
    static INPUT : &[u8;16] = &[0;16];
    static KEY: &[u8;16] = &[0;16];
    static NONCE : &[u8; 16] = &[1;16];

    let mut output_1 : [u8;20] =[0;20];
    let mut output_2 : [u8;20] =[0;20];
    let mut output_3 : [u8;20] =[0;20];

    //c.bench_function("Classic Encrypt", move |b| b.iter(|| encrypt(&input, &key)));
    //c.bench_function("x8 Encrypt", move |b| b.iter(|| encrypt_x8(&input_x8, &key)));
    c.bench(
        "Aes encryption",
        Benchmark::new(
            "Ctr encryption",
            move |b| b.iter(|| ctr_encryption(CTR_INPUT, &mut output_1, KEY, NONCE)),
        ).throughput(Throughput::Bytes(CTR_INPUT.len() as u32)),
    );

    c.bench(
        "Aes encryption decomp",
        Benchmark::new(
            "Ctr encryption decomp",
            move |b| b.iter(|| ctr_encryption_decompose(CTR_INPUT, &mut output_2, KEY, NONCE)),
        ).throughput(Throughput::Bytes(CTR_INPUT.len() as u32)),
    );

    c.bench(
        "Advance aes ctr encryption decomposition",
        Benchmark::new(
            "Advance Ctr encryption decomposition",
            move |b| b.iter(|| advance_ctr_encryption_decompose(CTR_INPUT, &mut output_3, KEY, NONCE)),
        ).throughput(Throughput::Bytes(CTR_INPUT.len() as u32)),
    );

    c.bench(
        "throughput Encrypt",
        Benchmark::new(
            "classic Encrypt",
            |b| b.iter(|| encrypt(INPUT, KEY)),
        ).throughput(Throughput::Bytes(INPUT.len() as u32)),
    );

}

fn encrypt(input : &[u8], key : &[u8]) -> [u8;16] {
    let mut output =  [0u8;16];
    //initialize the Encryptor
    let encryptor = AesSafe128Encryptor::new(&key);
    encryptor.encrypt_block(&input, &mut output);
    return output
}


fn construct_ctr_x8(in_ctr: &[u8], out_ctr_x8: &mut [u8]) {
    for (i, ctr_i) in out_ctr_x8.chunks_mut(in_ctr.len()).enumerate() {
        copy_memory(in_ctr, ctr_i);
        add_ctr(ctr_i, i as u8);
    }
}

fn add_ctr(ctr: &mut [u8], mut ammount: u8) {
    for i in ctr.iter_mut().rev() {
        let prev = *i;
        *i = i.wrapping_add(ammount);
        if *i >= prev {
            break;
        }
        ammount = 1;
    }
}

pub fn ctr_encryption(input : &[u8], mut output : &mut [u8], key : &[u8], nonce : &[u8]) {
    //initialize the Encryptor
    let mut _cipher = aes::ctr(KeySize::KeySize128, key, nonce);
    _cipher.process(input, &mut output);
}

pub fn ctr_encryption_decompose(input : &[u8], mut output : &mut [u8], key : &[u8], nonce : &[u8]) {
    let key_size;

    match key.len() {
        16 => {
            key_size = KeySize::KeySize128;
        }

        24 => {
            key_size = KeySize::KeySize192;
        }

        32 => {
            key_size = KeySize::KeySize256;
        }

        _ => {
            key_size = KeySize::KeySize128;
        }

    }

    match key_size {
            KeySize::KeySize128 => {
                let aes_dec = aessafe::AesSafe128EncryptorX8::new(key);
                let mut dec = Box::new(CtrModeX8::new(aes_dec, nonce));
                dec.process(input, &mut output);
            }
            KeySize::KeySize192 => {
                let aes_dec = aessafe::AesSafe192EncryptorX8::new(key);
                let mut dec = Box::new(CtrModeX8::new(aes_dec, nonce));
                dec.process(input, &mut output);
            }
            KeySize::KeySize256 => {
                let aes_dec = aessafe::AesSafe256EncryptorX8::new(key);
                let mut dec = Box::new(CtrModeX8::new(aes_dec, nonce));
                dec.process(input, &mut output);
            }
        }
}

pub fn advance_ctr_encryption_decompose(input : &[u8], output : &mut [u8], key : &[u8], nonce : &[u8]) {
    let aes_dec = aessafe::AesSafe128EncryptorX8::new(&key);

    //let dec = Box::new(CtrModeX8::new(aes_dec, iv));
    let block_size = aes_dec.block_size();
    let mut ctr_x8: Vec<u8> = repeat(0).take(block_size * 8).collect();
    //Add the counter to the nonce
    construct_ctr_x8(&nonce, &mut ctr_x8);

    let len = input.len();
    let mut i = 0;
    //capacity of bytes is 0
    let mut bytes = OwnedReadBuffer::new_with_len(repeat(0).take(aes_dec.block_size() * 8).collect(), 0);
    while i < len {
        //check if the offset position is at the end of the buffer capacity then encrypt the nonce/IV
        if bytes.is_empty() {
            let mut wb = bytes.borrow_write_buffer();
            aes_dec.encrypt_block_x8(&ctr_x8[..], wb.take_remaining());
            for ctr_i in &mut ctr_x8.chunks_mut(aes_dec.block_size()) {
                add_ctr(ctr_i, 8);
            }
        }

        let count = cmp::min(bytes.remaining(), len - i);

        let bytes_it = bytes.take_next(count).iter();
        let in_it = input[i..].iter();
        let out_it = &mut output[i..];

        for ((&x, &y), o) in bytes_it.zip(in_it).zip(out_it.iter_mut()) {
            *o = x ^ y;
        }

        i += count;
    }
}

//From cyptoutil.rs
fn copy_memory(src: &[u8], dst: &mut [u8]) {
    assert!(dst.len() >= src.len());
    unsafe {
        let srcp = src.as_ptr();
        let dstp = dst.as_mut_ptr();
        ptr::copy_nonoverlapping(srcp, dstp, src.len());
    }
}

criterion_group!(benches, ctr_benchmark);
criterion_main!(benches);
