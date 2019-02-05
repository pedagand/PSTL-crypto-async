mod rng;

fn main() {
    let f = rng::get_rand_float_0_and_1();
    let n = rng::get_rand_int_0_and_10();
    println!("random float between 0 and 1 {}",f);
    println!("random integer between 0 and 10 {}",n);


}
