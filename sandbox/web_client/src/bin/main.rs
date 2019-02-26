use std::io::prelude::*;
use std::mem::transmute;
use std::net::TcpStream;
use std::thread;

fn main() {
    let mut vec_thread = Vec::new();

    //Send 100 requests at the same time
    for _i in 0..100 {
        let handle = thread::spawn(move || {
            let mut stream = TcpStream::connect("127.0.0.1:7870").unwrap();

            //generates a u64 between 0 and 10
            let r: u64 = web_client::get_rand_u64();
            println!("random u64 generated  {}", r);
            let bytes: [u8; 8] = unsafe { transmute(r.to_be()) };

            //write
            stream.write(&bytes).unwrap();

            //read
            let mut buffer = [0; 8];
            stream.read(&mut buffer).unwrap();

            let num = u64::from_be_bytes(buffer);

            println!("Received : {}", num);
        });
        vec_thread.push(handle);
    }

    for t in vec_thread {
        t.join().unwrap();
    }
}
