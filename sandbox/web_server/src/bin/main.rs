use std::io::prelude::*;
use rand::prelude::*;
use std::net::{TcpStream, TcpListener};
use web_server::{ThreadPool, Scheduler};
use std::sync::Arc;
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
            submit_job(stream, scheduler, size);
        });
    }
}

pub fn submit_job(mut stream: TcpStream, scheduler: Arc<Scheduler>, size: usize) {
    let mut buffer = [0; 8];
    stream.read(&mut buffer).unwrap();
    let plain = u64::from_be_bytes(buffer);
    let key = rand::thread_rng().gen();

    let mut cpt = scheduler.counter_index.lock().unwrap();
    let index = *cpt;
    *cpt += 1;
    std::mem::drop(cpt);

    assert!(index >= 0 && index <= size as i32);
    let mut buff = scheduler.buffer.lock().unwrap();
    buff[index as usize].key = key;
    buff[index as usize].plain = plain;

    std::mem::drop(buff);

    if index == size as i32 - 1 {
        scheduler.chan_wait_to_encrypt.lock().unwrap().recv().unwrap();

        let mut buff = scheduler.buffer.lock().unwrap();
        let mut crypt_buffer = scheduler.crypt_buff.lock().unwrap();
        for i in 0..(size) {
            crypt_buffer[i] = buff[i].plain ^ buff[i].key;
            thread::sleep(time::Duration::from_millis(1));
        }
        let result = crypt_buffer[index as usize];
        std::mem::drop(buff);
        std::mem::drop(crypt_buffer);
        stream.write(&u64_to_array_of_u8(result)).unwrap();
        let mut cpt = scheduler.counter_index.lock().unwrap();
        *cpt = 0;
        std::mem::drop(cpt);
        for _ in 0..size - 1 {
            scheduler.chan_ok_to_read.lock().unwrap().send(()).unwrap();
        }
    } else {
        let mut c_wait = scheduler.counter_wait.lock().unwrap();
        *c_wait += 1;
        if *c_wait == (size as i32) - 1 {
            scheduler.chan_ok_to_encrypt.lock().unwrap().send(()).unwrap();
            *c_wait = 0;
        }
        std::mem::drop(c_wait);
        scheduler.chan_wait_to_read.lock().unwrap().recv().unwrap();
        let mut crypt_buffer = scheduler.crypt_buff.lock().unwrap();
        let result = crypt_buffer[index as usize];
        std::mem::drop(crypt_buffer);
        assert!(result == plain ^ key);
        stream.write(&u64_to_array_of_u8(result)).unwrap();
    }
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