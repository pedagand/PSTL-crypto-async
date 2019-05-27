use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use rand::Rng;
use std::env;
fn main() {
    let mut vec_thread = Vec::new();
    let args: Vec<String> = env::args().collect();
    let nb_request: usize = args[1].parse().unwrap();

    for _i in 0..nb_request {
        let handle = thread::spawn(move || {
            let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();

            let mut rng = rand::thread_rng();
            let r: [u8; 8] = rng.gen();

            stream.write(&r).unwrap();

            //read
            let mut buffer = [0; 8];
            stream.read(&mut buffer).unwrap();

            let num = u64::from_be_bytes(buffer);
            println!("Received : {}", num);

            // Check that we are getting back the same data
        });
        vec_thread.push(handle);
    }

    for t in vec_thread {
        t.join().unwrap();
    }
}
