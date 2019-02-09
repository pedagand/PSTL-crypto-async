extern crate trans_func;

#[cfg(test)]
mod tests {



    //test with a table who has 1 in the last column
    #[test]
    fn test_ortho1() {
        let mut data: [u64; 64] = [0x0000000000000001; 64];
        let mut out: [u64; 64] = [0x0000000000000000; 64];

        trans_func::orthogonalize(&mut data, &mut out);
        for i in 0..63 {
            assert_eq!(out[i], 0x0000000000000000);
        }

        assert_eq!(out[63], 0xFFFFFFFFFFFFFFFF);
    }



    //test with a table who has 1 in the first column
    #[test]
    fn test_ortho2() {
        let mut data: [u64; 64] = [0x1000000000000000; 64];
        let mut out: [u64; 64] = [0x0000000000000000; 64];

        trans_func::orthogonalize(&mut data, &mut out);

        assert_eq!(out[0], 0x0000000000000000);
        assert_eq!(out[1], 0x0000000000000000);
        assert_eq!(out[2], 0x0000000000000000);
        assert_eq!(out[3], 0xFFFFFFFFFFFFFFFF);

        for i in 4..64 {
            assert_eq!(out[i], 0x0000000000000000);
        }
    }


    //test with a table who has 1 in all 64 column
    #[test]
    fn test_ortho3() {
        let mut data: [u64; 64] = [0xFFFFFFFFFFFFFFFF; 64];
        let mut out: [u64; 64] = [0x0000000000000000; 64];

        trans_func::orthogonalize(&mut data, &mut out);

        for i in 0..64 {
            assert_eq!(out[i], 0xFFFFFFFFFFFFFFFF);
        }
    }

}
