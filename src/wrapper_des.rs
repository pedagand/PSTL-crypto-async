extern "C" {
    fn DES(plain: &[u64; 64], key: &[u64; 56], cipher: &mut [u64; 64]);
}

pub fn des(plain: [u64; 64], key: [u64; 56], cipher: &mut [u64; 64]) {
    unsafe {
        DES(&plain, &key, cipher);
    }
}
