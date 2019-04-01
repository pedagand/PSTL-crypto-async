extern crate crypto;


use crate::crypto::symmetriccipher::BlockEncryptorX8;
use crate::crypto::buffer::WriteBuffer;
use crate::crypto::buffer::ReadBuffer;
use crypto::aes::{self, KeySize};
use crypto::symmetriccipher::SynchronousStreamCipher;
use std::iter::repeat;
use crypto::blockmodes::CtrModeX8;
use crypto::aessafe;
use crypto::buffer::OwnedReadBuffer;
use rand::{OsRng, Rng};
use std::cmp;


fn main() {
    let input : [u8;16] = [0;16];
    let mut key: Vec<u8> = repeat(0u8).take(16).collect();
    let mut random = OsRng::new().expect("Failed to get OS random generator");
    random.fill_bytes(&mut key[..]);

    let _ctr_input : [u8;20] = [1;20];
    let _ctr_input_2 : [u8;20] = [1;20];
    let mut output : [u8;20] =[0;20];
    let mut output_2 : [u8;20] =[0;20];
    let mut output_3 : [u8;20] =[0;20];
    let mut nonce: Vec<u8> = repeat(0u8).take(16).collect();
    random.fill_bytes(&mut nonce[..]);

    let aes_dec = aessafe::AesSafe128EncryptorX8::new(&key);
    //let mut dec = Box::new(CtrModeX8::new(aes_dec, &nonce));

    //let mut ctr_x8 = CtrModeX8::new(aes_dec, &nonce);
    let len = _ctr_input.len();
    let mut i = 0;
    let mut bytes = OwnedReadBuffer::new_with_len(repeat(0).take(aes_dec.block_size() * 8).collect(), 0);
    println!("Start of the loop");
    while i < len {
        println!("i : {}", i);
        if bytes.is_empty() {
            let mut wb = bytes.borrow_write_buffer();
            aes_dec.encrypt_block_x8(&_ctr_input, wb.take_remaining());
            for ctr_i in &mut nonce.chunks_mut(aes_dec.block_size()) {
                add_ctr(ctr_i, 8);
            }
        }
        let count = cmp::min(bytes.remaining(), len - i);
        let bytes_it = bytes.take_next(count).iter();
        let in_it = _ctr_input[i..].iter();
        let out_it = &mut output_3[i..];
        for ((&x, &y), o) in bytes_it.zip(in_it).zip(out_it.iter_mut()) {
            *o = x ^ y;
        }
        i += count;
    }

    //ctr_x8.process(&_ctr_input, &mut output_3);



    ctr_encryption(&_ctr_input, &mut output, &key, &nonce);
    ctr_encryption_decompose(&_ctr_input_2, &mut output_2, &key, &nonce);
    println!("1 : {:?}", output);
    println!("2 : {:?}", output_2);
    println!("3 : {:?}", output_3);
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


fn ctr_encryption(input : &[u8], mut output : &mut [u8], key : &[u8], nonce : &[u8]) {
    //initialize the Encryptor
    let mut _cipher = aes::ctr(KeySize::KeySize128, key, nonce);
    _cipher.process(input, &mut output);
}

fn ctr_encryption_decompose(input : &[u8], mut output : &mut [u8], key : &[u8], nonce : &[u8]) {
    let mut key_size = KeySize::KeySize128;

    match key.len() {
        16 => {
            key_size = KeySize::KeySize128;
            println!{"Size is 128"}
        }

        24 => {
            key_size = KeySize::KeySize192;
            println!{"Size is 192"}
        }

        32 => {
            key_size = KeySize::KeySize256;
            println!{"Size is 256"}
        }

        _ => {
            key_size = KeySize::KeySize128;
            println!{"Size is 128, because of non exhaustive"}
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
