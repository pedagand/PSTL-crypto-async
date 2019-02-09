static MASK_L : [u64; 6] =  [
	0xaaaaaaaaaaaaaaaa,
	0xcccccccccccccccc,
	0xf0f0f0f0f0f0f0f0,
	0xff00ff00ff00ff00,
	0xffff0000ffff0000,
       0xffffffff00000000];

static MASK_R : [u64; 6] =  [
	0x5555555555555555,
	0x3333333333333333,
	0x0f0f0f0f0f0f0f0f,
	0x00ff00ff00ff00ff,
	0x0000ffff0000ffff,
       0x00000000ffffffff];

pub fn transpose(data : &mut [u64;64]) {
        for i in 0..6 {
            let n = 1 << i;
            for j in (0..64).step_by(2*n) {
                for k in 0..n {
                    let u : u64 = data[j+k] & MASK_L[i];
                    let v : u64 = data[j+k] & MASK_R[i];
                    let x : u64 = data[j+n+k] & MASK_L[i];
                    let y : u64 = data[j+n+k] & MASK_R[i];

                    data[j+k] = u |(x >> n);
                    data[j+n+k] = (v << n) | y;
                }
            }
        }
}
