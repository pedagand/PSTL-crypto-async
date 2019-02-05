static MASK_L: [u64; 6] = [
    0xaaaaaaaaaaaaaaaa,
    0xcccccccccccccccc,
    0xf0f0f0f0f0f0f0f0,
    0xff00ff00ff00ff00,
    0xffff0000ffff0000,
    0xffffffff00000000,
];

static MASK_R: [u64; 6] = [
    0x5555555555555555,
    0x3333333333333333,
    0x0f0f0f0f0f0f0f0f,
    0x00ff00ff00ff00ff,
    0x0000ffff0000ffff,
    0x00000000ffffffff,
];

pub fn real_ortho(data: &mut [u64; 64]) {
    let mut i = 0;
    while i < 6 {
        //1UL est represente par 8 octet en C
        let m = 1u64 << i;
        let n = m as usize;

        let mut j = 0 as usize;
        while j < 64 {
            let mut k = 0;
            while k < n {
                let u: u64 = data[j + k] & MASK_L[i];
                let v: u64 = data[j + k] & MASK_R[i];
                let x: u64 = data[j + n + k] & MASK_L[i];
                let y: u64 = data[j + n + k] & MASK_R[i];
                data[j + k] = u | (x >> n);
                data[j + n + k] = (v << n) | y;
                k = k + 1;
            }
            j += 2 * n;
        }
        i += 1;
    }
}