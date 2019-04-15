use rand::prelude::*;
use std::sync::{mpsc, Arc, Mutex, Barrier};
use std::{thread, time};
use web_server::{Cell, ThreadPool};
use std::env;

extern crate web_server;


fn execute(threadpool: Arc<Mutex<ThreadPool>>, size: usize, vbuff: Arc<Mutex<Vec<Cell>>>, counter: Arc<Mutex<i32>>,
           counter_write: Arc<Mutex<i32>>, sender: Arc<Mutex<mpsc::Sender<()>>>, receiver: Arc<Mutex<mpsc::Receiver<()>>>
           , r: Arc<Mutex<mpsc::Receiver<()>>>, s: Arc<Mutex<mpsc::Sender<()>>>,
           lock_plain: Arc<Mutex<[u64; 64]>>, lock_key: Arc<Mutex<[u64; 64]>>,
           rd: Arc<Mutex<mpsc::Receiver<()>>>,
           sd: Arc<Mutex<mpsc::Sender<()>>>,
           counter_wait: Arc<Mutex<i32>>, bench_size: usize,
) {
    let args: Vec<String> = env::args().collect();
    let nb_request = args[2].parse().unwrap();
    let barrier = Arc::new(Barrier::new(bench_size + 1));
    let pool = threadpool.lock().unwrap();

    for i in 0..nb_request {
        let sender = Arc::clone(&sender);
        let receiver = Arc::clone(&receiver);
        let counter = Arc::clone(&counter);
        let r = Arc::clone(&r);
        let s = Arc::clone(&s);
        let counter_write = Arc::clone(&counter_write);

        let rd = Arc::clone(&rd);
        let sd = Arc::clone(&sd);
        let counter_wait = Arc::clone(&counter_wait);


        let _plain = lock_plain.lock().unwrap();
        let plain = _plain[i];
        std::mem::drop(_plain);

        let _key = lock_key.lock().unwrap();
        let key = _key[i];
        std::mem::drop(_key);

        let k = Arc::new(Mutex::new(key));
        let p = Arc::new(Mutex::new(plain));

        let plain = Arc::clone(&p);
        let key = Arc::clone(&k);
        let buff = Arc::clone(&vbuff);
        let c = barrier.clone();
        pool.execute(move || {
            //  ln!("dans execute");
            handle_connection(counter, buff, sender, receiver, size, counter_write, s, r, plain, key, rd, sd, counter_wait);
            c.wait();
            println!("hello");

        });
    }
    println!("hello");

    barrier.wait();
}

pub fn handle_connection(
    counter: Arc<Mutex<i32>>,
    buf: Arc<Mutex<Vec<Cell>>>,
    sen: Arc<Mutex<mpsc::Sender<()>>>,
    rec: Arc<Mutex<mpsc::Receiver<()>>>,
    size: usize,
    counter_write: Arc<Mutex<i32>>,
    s: Arc<Mutex<mpsc::Sender<()>>>,
    r: Arc<Mutex<mpsc::Receiver<()>>>,
    lock_plain: Arc<Mutex<u64>>,
    lock_key: Arc<Mutex<u64>>,
    rd: Arc<Mutex<mpsc::Receiver<()>>>,
    sd: Arc<Mutex<mpsc::Sender<()>>>,
    counter_wait: Arc<Mutex<i32>>,
) {
    let mut cpt = counter.lock().unwrap();
    if *cpt == -1 {
        if size != 1 {
            r.lock().unwrap().recv().unwrap();
        }
        *cpt = 0;
    }
    let index = *cpt;

    *cpt += 1;
    std::mem::drop(cpt);
    assert!(index >= 0 && index <= size as i32);
    let mut buff = buf.lock().unwrap();

    let plain = lock_plain.lock().unwrap();

    let key = lock_key.lock().unwrap();

    buff[index as usize].key = *key;
    buff[index as usize].plain = *plain;
    let local_plain = *plain;
    let local_key = *key;

    std::mem::drop(buff);
    std::mem::drop(plain);

    std::mem::drop(key);

    if index == size as i32 - 1 {
        rd.lock().unwrap().recv().unwrap();
        let mut buff = buf.lock().unwrap();
        for i in 0..(size) {
            buff[i].plain ^= buff[i].key;
            thread::sleep(time::Duration::from_millis(1));
        }
        let result = buff[index as usize].plain;
        std::mem::drop(buff);

        let mut cpt = counter.lock().unwrap();
        *cpt = -1;
            let _sender = sen.lock().unwrap().send(()).unwrap();
        }
        std::mem::drop(cpt);
    } else {
        let mut c_wait = counter_wait.lock().unwrap();
        *c_wait += 1;
        if *c_wait == (size as i32) - 1 {
            sd.lock().unwrap().send(()).unwrap();
            *c_wait = 0;
        }
        std::mem::drop(c_wait);
        let _received = rec.lock().unwrap().recv().unwrap();
        let buff = buf.lock().unwrap();
        let result = buff[index as usize].plain;
        assert!(result == local_plain ^ local_key);
        //   ln!("res {} index {}",result,index);
        let mut c = counter_write.lock().unwrap();
        *c += 1;
        if *c == (size as i32) - 1 {
            let _sender = s.lock().unwrap().send(()).unwrap();
            *c = 0;
        }
        std::mem::drop(buff);
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let bench_size = args[1].parse().unwrap();
    let pool = Arc::new(Mutex::new(ThreadPool::new(bench_size)));
    let size: usize = 32;
    let vec: Vec<Cell> = vec![Cell { plain: 0, key: 0 }; size];
    let buf: Arc<Mutex<Vec<Cell>>> = Arc::new(Mutex::new(vec));
    let counter = Arc::new(Mutex::new(0));
    let counter_write = Arc::new(Mutex::new(0));
    let (sender, receiver) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));
    let sender = Arc::new(Mutex::new(sender.clone()));
    let (s, r) = mpsc::channel();
    let r = Arc::new(Mutex::new(r));
    let s = Arc::new(Mutex::new(s.clone()));

    let (sd, rd) = mpsc::channel();
    let rd = Arc::new(Mutex::new(rd));
    let sd = Arc::new(Mutex::new(sd.clone()));
    let counter_wait = Arc::new(Mutex::new(0));


    let mut plain: [u64; 64] = [0; 64];
    let mut key: [u64; 64] = [0; 64];
    for i in 0..64 {
        plain[i] = rand::thread_rng().gen();
        key[i] = rand::thread_rng().gen();
    }
    let plain: Arc<Mutex<[u64; 64]>> = Arc::new(Mutex::new(plain));
    let key: Arc<Mutex<[u64; 64]>> = Arc::new(Mutex::new(key));

    for i in 0..50 {
        // ln!("i {}",i);

        execute(Arc::clone(&pool), size,
                Arc::clone(&buf), Arc::clone(&counter),
                Arc::clone(&counter_write), Arc::clone(&sender),
                Arc::clone(&receiver), Arc::clone(&r), Arc::clone(&s),
                Arc::clone(&plain), Arc::clone(&key),
                Arc::clone(&rd), Arc::clone(&sd), Arc::clone(&counter_wait), bench_size,
        );
    }
}
