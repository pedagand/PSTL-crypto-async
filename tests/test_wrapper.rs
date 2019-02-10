extern crate crypto_async;

#[cfg(test)]
mod tests {

    #[test]
    fn test_wrapper() {
        let mut plain: [u64; 64] = [0x0000000000000000; 64];
        let mut key: [u64; 56] = [0x0000000000000000; 56];
        let mut cipher: [u64; 64] = [0x0000000000000000; 64];

        for i in 0..64 {
            plain[i as usize] = 13 * i;
        }

        for i in 0..56 {
            key[i as usize] = 7 * i;
        }

        crypto_async::wrapper_des::des(plain, key, &mut cipher);

        for i in 0..64 {
            assert_eq!(cipher[i], (plain[i] ^ key[i % 56]));
        }
    }

}
