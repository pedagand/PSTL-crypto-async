use std::io::prelude::*;
use std::mem::transmute;
use std::net::TcpStream;
use std::thread;
use std::time::Instant;
use std::time::Duration;

fn main() {
    let mut vec_thread = Vec::new();
    let mut vec_data = Vec::new();
    //Send 100 requests at the same time
    for _i in 0..40 {
        let handle = thread::spawn(move || {
            let mut stream = TcpStream::connect("127.0.0.1:7870").unwrap();

            //generates a u64 between 0 and u64_MAX
            let r: u64 = web_client::get_rand_u64();
            // println!("random u64 generated  {}", r);
            let bytes: [u8; 8] = unsafe { transmute(r.to_be()) };


            let start = Instant::now();
            //write
            stream.write(&bytes).unwrap();

            //read
            let mut buffer = [0; 8];
            stream.read(&mut buffer).unwrap();

            let duration = start.elapsed();
            println!("Time elapsed is {:?}",duration );

            let num = u64::from_be_bytes(buffer);

            // println!("Received : {}", num);
            duration
        });
        vec_thread.push(handle);
    }

    for t in vec_thread {
        let d = t.join().unwrap();
        vec_data.push(d);
    }

}
