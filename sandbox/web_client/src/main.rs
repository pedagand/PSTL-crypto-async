use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
fn main() {
    let mut vec_thread = Vec::new();

    //Send 100 requests at the same time
    for _i in 0..100 {
        let handle = thread::spawn(move || {
            let mut stream = TcpStream::connect("127.0.0.1:7870").unwrap();

            //write
            let request = "hello";
            stream.write(&request.as_bytes()).unwrap();

            //read
            let mut buffer = [0; 512];
            stream.read(&mut buffer).unwrap();

            println!("Received : {}", String::from_utf8_lossy(&buffer));
        });
        vec_thread.push(handle);
    }

    for t in vec_thread {
        t.join().unwrap();
    }
}
