use rand::prelude::*;

pub fn get_rand_float_0_and_1() -> f64 {
    let mut rng = rand::thread_rng();
    let y: f64 = rng.gen(); // generates a float between 0 and 1
    return y;
}

pub fn get_rand_int_0_and_10() -> i32 {
    let mut rng = rand::thread_rng();
    let y = rng.gen_range(0, 10);
    return y;
}