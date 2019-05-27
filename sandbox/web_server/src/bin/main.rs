use std::io::prelude::*;
use rand::prelude::*;
use std::net::{TcpStream, TcpListener};
use web_server::{ThreadPool, Scheduler, submit_job};
use std::sync::{Arc, Mutex};
use std::{thread, time, env};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let args: Vec<String> = env::args().collect();
    let size: usize = args[1].parse().unwrap();
    let pool = ThreadPool::new(size);
    let scheduler = Scheduler::new(size);
    let scheduler = Arc::new(scheduler);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let scheduler = Arc::clone(&scheduler);
        pool.execute(move || {
            handle_connection(stream, scheduler, size);
        });
    }
}


pub fn handle_connection(mut stream: TcpStream, scheduler: Arc<Scheduler>, size: usize) {
    let mut buffer = [0; 8];
    stream.read(&mut buffer).unwrap();
    let plain = u64::from_be_bytes(buffer);
    let key = rand::thread_rng().gen();

    let lock_plain = Arc::new(Mutex::new(plain));
    let lock_key = Arc::new(Mutex::new(key));
    let resultIndex = submit_job(Arc::clone(&scheduler), size, lock_plain, lock_key);
    stream.write(&u64_to_array_of_u8(resultIndex.result)).unwrap();

}

fn u64_to_array_of_u8(x: u64) -> [u8; 8] {
    let b1: u8 = ((x >> 56) & 0xff) as u8;
    let b2: u8 = ((x >> 48) & 0xff) as u8;
    let b3: u8 = ((x >> 40) & 0xff) as u8;
    let b4: u8 = ((x >> 32) & 0xff) as u8;
    let b5: u8 = ((x >> 24) & 0xff) as u8;
    let b6: u8 = ((x >> 16) & 0xff) as u8;
    let b7: u8 = ((x >> 8) & 0xff) as u8;
    let b8: u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4, b5, b6, b7, b8];
}