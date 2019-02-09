extern crate crypto_async;

#[cfg(test)]
mod tests {

    //test with a table who has 1 in the last column
    #[test]
    fn test_transpose1() {
        let mut data: [u64; 64] = [0x0000000000000001; 64];

        crypto_async::transposition::transpose(&mut data);
        for i in 0..63 {
            assert_eq!(data[i], 0x0000000000000000);
        }

        assert_eq!(data[63], 0xFFFFFFFFFFFFFFFF);
    }

    //test with a table who has 1 in the first column
    #[test]
    fn test_transpose2() {
        let mut data: [u64; 64] = [0x1000000000000000; 64];

        crypto_async::transposition::transpose(&mut data);

        assert_eq!(data[0], 0x0000000000000000);
        assert_eq!(data[1], 0x0000000000000000);
        assert_eq!(data[2], 0x0000000000000000);
        assert_eq!(data[3], 0xFFFFFFFFFFFFFFFF);

        for i in 4..64 {
            assert_eq!(data[i], 0x0000000000000000);
        }
    }

    //test with a table who has 1 in all 64 column
    #[test]
    fn test_transpose3() {
        let mut data: [u64; 64] = [0xFFFFFFFFFFFFFFFF; 64];

        crypto_async::transposition::transpose(&mut data);

        for i in 0..64 {
            assert_eq!(data[i], 0xFFFFFFFFFFFFFFFF);
        }
    }

}
