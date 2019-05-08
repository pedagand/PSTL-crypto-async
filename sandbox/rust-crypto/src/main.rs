extern crate crypto;
extern crate rustc_serialize;

use rand::{OsRng, Rng};
use crypto::symmetriccipher::{BlockEncryptor, BlockDecryptor, BlockEncryptorX8, BlockDecryptorX8};
use crypto::aessafe::{AesSafe128Encryptor, AesSafe128Decryptor, AesSafe128EncryptorX8, AesSafe128DecryptorX8};
use std::iter::repeat;
use rustc_serialize::base64::*;
use crypto::aes::{self, KeySize};

fn main() {
    //creation of a random generator
    let mut random = OsRng::new().expect("Failed to get OS random generator");
    let mut key: Vec<u8> = repeat(0u8).take(16).collect();
    //fill the key with random unsigned int value
    random.fill_bytes(&mut key[..]);

    //Exemple of encryption and decryption
    let input : [u8;16] = [0;16];
    let input_x8 : [u8;128] = [0;128];
    //to_base64 is to have a better representation of the array
    println!("Clé : {:?}", &key.to_base64(STANDARD));
    println!("Message : {:?}", input.to_base64(STANDARD));
    let crypter : [u8;16] = encrypt(&input, &key);
    let decrypter : [u8;16] = decrypyt(&crypter, &key);
    //Print of the simple encryption and decryption
    println!("Cryptage : {:?}", crypter.to_base64(STANDARD));
    println!("Decryptage : {:?}", decrypter.to_base64(STANDARD));
    //Print of the X8 encryption and decryption
    let crypter : [u8;128] = encrypt_x8(&input_x8, &key);
    let decrypter : [u8;128] = decrypyt_x8(&crypter, &key);
    println!("Cryptage 8 : {:?}", crypter.to_base64(STANDARD));
    println!("Decryptage 8 : {:?}", decrypter.to_base64(STANDARD));

    let mut ctr_val : Vec<u8> = repeat(0u8).take(16).collect();
    random.fill_bytes(&mut ctr_val[..]);

    let _ctr_input : [u8;16] = [1;16];
    let mut output : [u8;16] =[0;16];
    let mut nonce: Vec<u8> = repeat(0u8).take(16).collect();
    random.fill_bytes(&mut nonce[..]);

    ctr_encryption(&input, &mut output, &key, &nonce);

    println!("{:?}", output);
}

/* Encrypt a array of unsigned int (128 bits)
return the encrypted array as an array of unsigned int
*/
fn encrypt(input : &[u8], key : &[u8]) -> [u8;16] {
    let mut output =  [0u8;16];
    //initialize the Encryptor
    let encryptor = AesSafe128Encryptor::new(&key);
    encryptor.encrypt_block(&input, &mut output);
    return output
}

/* Decrypt a array of unsigned int (1 block of 128 bits)
return the decrypted array as an array of unsigned int
*/
fn decrypyt(input : &[u8], key : &[u8]) -> [u8;16] {
    let mut output =  [0u8;16];
    //initialize the Decryptor
    let decryptor = AesSafe128Decryptor::new(&key);
    decryptor.decrypt_block(&input, &mut output);
    return output
}

/* Encrypt a array of unsigned int (128 bytes = 8 block of 128 bits)
return the encrypted array as an array of unsigned int
*/
fn encrypt_x8(input : &[u8], key : &[u8]) -> [u8;128] {
    let mut output =  [1u8;128];
    //initialize the Encryptor
    let encryptor = AesSafe128EncryptorX8::new(&key);
    encryptor.encrypt_block_x8(&input, &mut output);
    return output
}

/* Decrypt a array of unsigned int (128 bytes = 8 block of 128 bits)
return the decrypted array as an array of unsigned int
*/
fn decrypyt_x8(input : &[u8], key : &[u8]) -> [u8;128] {
    let mut output =  [1u8;128];
    //initialize the Decryptor
    let decryptor = AesSafe128DecryptorX8::new(&key);
    decryptor.decrypt_block_x8(&input, &mut output);
    return output
}

fn ctr_encryption(input : &[u8], output : &mut [u8], key : &[u8], nonce : &[u8]) {
    //initialize the Encryptor
    let mut _cipher = aes::ctr(KeySize::KeySize128, key, nonce);
    _cipher.process(input, &mut output[..]);
}
