use rand::prelude::*;
use std::sync::{mpsc, Arc, Mutex, Barrier};
use threaded::{Cell, ThreadPool, Scheduler};

extern crate criterion;
extern crate threaded;

use criterion::*;


fn init(threadpool: Arc<Mutex<ThreadPool>>, buff: Arc<Mutex<Vec<Cell>>>, size: usize, scheduler: Arc<Scheduler>,
        lock_plain: Arc<Mutex<[u64; 64]>>, lock_key: Arc<Mutex<[u64; 64]>>, bench_size: usize,
) {
    let barrier = Arc::new(Barrier::new(bench_size + 1));
    let pool = threadpool.lock().unwrap();

    for i in 0..64{
        let _plain = lock_plain.lock().unwrap();
        let plain = _plain[i];
        std::mem::drop(_plain);

        let _key = lock_key.lock().unwrap();
        let key = _key[i];
        std::mem::drop(_key);

        let k = Arc::new(Mutex::new(key));
        let p = Arc::new(Mutex::new(plain));
        let scheduler = Arc::clone(&scheduler);
        let plain = Arc::clone(&p);
        let key = Arc::clone(&k);
        let buffer = Arc::clone(&buff);
        let c = barrier.clone();

        pool.execute(move || {
            threaded::runtime::demo_submit_job(buffer, scheduler, size, key, plain);
            c.wait();
        });
    }
    barrier.wait();
}

fn bench(c: &mut Criterion) {
    let bench_size = 64;
    let pool = Arc::new(Mutex::new(ThreadPool::new(bench_size)));
    let size: usize = bench_size;
    let vec: Vec<Cell> = vec![Cell { plain: 0, key: 0 }; size];
    let buf: Arc<Mutex<Vec<Cell>>> = Arc::new(Mutex::new(vec));

    let counter_index = Arc::new(Mutex::new(0));
    let counter_write = Arc::new(Mutex::new(0));
    let (chan_ok_to_read, chan_wait_to_read) = mpsc::channel();
    let chan_wait_to_read = Arc::new(Mutex::new(chan_wait_to_read));
    let chan_ok_to_read = Arc::new(Mutex::new(chan_ok_to_read.clone()));
    let (chan_ok_to_write, chan_wait_to_write) = mpsc::channel();
    let chan_wait_to_write = Arc::new(Mutex::new(chan_wait_to_write));
    let chan_ok_to_write = Arc::new(Mutex::new(chan_ok_to_write.clone()));

    let (chan_ok_to_encrypt, chan_wait_to_encrypt) = mpsc::channel();
    let chan_wait_to_encrypt = Arc::new(Mutex::new(chan_wait_to_encrypt));
    let chan_ok_to_encrypt = Arc::new(Mutex::new(chan_ok_to_encrypt.clone()));
    let counter_wait = Arc::new(Mutex::new(0));


    let mut plain: [u64; 64] = [0; 64];
    let mut key: [u64; 64] = [0; 64];
    for i in 0..64 {
        plain[i] = rand::thread_rng().gen();
        key[i] = rand::thread_rng().gen();
    }
    let plain: Arc<Mutex<[u64; 64]>> = Arc::new(Mutex::new(plain));
    let key: Arc<Mutex<[u64; 64]>> = Arc::new(Mutex::new(key));

    let scheduler: Scheduler = Scheduler {
        chan_wait_to_write,
        chan_ok_to_write,
        chan_wait_to_encrypt,
        chan_ok_to_encrypt,
        chan_wait_to_read,
        chan_ok_to_read,
        counter_index,
        counter_wait,
        counter_write,
    };

    let scheduler = Arc::new(scheduler);

    c.bench(
        "server",
        Benchmark::new("server", move |b| b.iter(|| init(Arc::clone(&pool),
                                                         Arc::clone(&buf), size,
                                                         Arc::clone(&scheduler),
                                                         Arc::clone(&plain),
                                                         Arc::clone(&key), bench_size)
        ),
        ).throughput(Throughput::Bytes(8)), );
}

criterion_group!(benches, bench);
criterion_main!(benches);
