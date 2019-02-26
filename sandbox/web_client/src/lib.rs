use rand::prelude::*;

pub fn get_rand_u64() -> u64 {
    let mut rng = rand::thread_rng();
    let y: u64 = rng.gen_range(0, std::u64::MAX); // generates a u64 between 0 and u64_MAX
    return y;
}
