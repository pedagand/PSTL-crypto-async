use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use web_server::{ThreadPool, Cell};
use std::sync::{Arc, Mutex, mpsc};
use rand::prelude::*;

const SIZE: usize = 64;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let buffer = Arc::new(Mutex::new([Cell { plain: 0, key: 0 }; SIZE]));

    let counter = Arc::new(Mutex::new(1));
    let pool = ThreadPool::new(SIZE);
    let (sender, receiver) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));
    let sender = Arc::new(Mutex::new(sender.clone()));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let sender = Arc::clone(&sender);
        let receiver = Arc::clone(&receiver);
        let counter = Arc::clone(&counter);
        let buffer = Arc::clone(&buffer);
        pool.execute(move || {
            handle_connection(stream, counter, buffer, sender, receiver);
        });
    }
}


pub fn handle_connection(mut stream: TcpStream, counter: Arc<Mutex<i32>>, buf: Arc<Mutex<[Cell; SIZE]>>, sen: Arc<Mutex<mpsc::Sender<()>>>, rec: Arc<Mutex<mpsc::Receiver<()>>>) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    //get the index of the first nullbyte
    let mut j = 0;
    for i in 0..512 {
        if buffer[i] == 0 {
            break;
        } else { j += 1; }
    }
    let plain_string: String = String::from_utf8_lossy(&buffer[..j]).to_string();

    let plain: u64 = plain_string.parse().unwrap();
    let key = rand::thread_rng().gen();

    let mut cpt = counter.lock().unwrap();
    let number = *cpt;
    let mut buff = buf.lock().unwrap();
    let index = (number - 1) % ((SIZE as i32));
    buff[index as usize].key = key;
    buff[index as usize].plain = plain;
    std::mem::drop(buff);
    *cpt += 1;
    std::mem::drop(cpt);
    if (number % SIZE as i32) == 0 {
        let mut buff = buf.lock().unwrap();
        let result = key;
        for i in 0..(SIZE) {
            buff[i].key ^= result;
        }
        let res: String = buff[SIZE - 1].to_string();
        stream.write(res.as_bytes()).unwrap();
        std::mem::drop(buff);
        let mut cpt = counter.lock().unwrap();
        *cpt = 1;
        for _ in 0..(SIZE - 1) {
            let _sender = sen.lock().unwrap().send(()).unwrap();
        }
    } else {
        let _received = rec.lock().unwrap().recv().unwrap();
        let buff = buf.lock().unwrap();
        let index = (number - 1) % ((SIZE as i32) - 1);
        let result = buff[index as usize];
        stream.write(result.to_string().as_bytes()).unwrap();
        std::mem::drop(buff);
    }
}