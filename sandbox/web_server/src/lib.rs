use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;


pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

pub struct Scheduler {
    pub chan_wait_to_write: Arc<Mutex<mpsc::Receiver<()>>>,
    pub chan_ok_to_write: Arc<Mutex<mpsc::Sender<()>>>,
    pub chan_wait_to_encrypt: Arc<Mutex<mpsc::Receiver<()>>>,
    pub chan_ok_to_encrypt: Arc<Mutex<mpsc::Sender<()>>>,
    pub chan_wait_to_read: Arc<Mutex<mpsc::Receiver<()>>>,
    pub chan_ok_to_read: Arc<Mutex<mpsc::Sender<()>>>,
    pub counter_index: Arc<Mutex<i32>>,
    pub counter_wait: Arc<Mutex<i32>>,
    pub counter_write: Arc<Mutex<i32>>,
    pub buffer: Arc<Mutex<Vec<Cell>>>,
    pub crypt_buff: Arc<Mutex<Vec<u64>>>,
}

impl Scheduler {
    pub fn new(size: usize) -> Scheduler {
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
        let buffer: Arc<Mutex<Vec<Cell>>> = Arc::new(Mutex::new(vec![Cell { plain: 0, key: 0 }; size]));
        let crypt_buff: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(vec![0; size]));

        Scheduler {
            chan_wait_to_write,
            chan_ok_to_write,
            chan_wait_to_encrypt,
            chan_ok_to_encrypt,
            chan_wait_to_read,
            chan_ok_to_read,
            counter_index,
            counter_wait,
            counter_write,
            buffer,
            crypt_buff,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Cell {
    pub plain: u64,
    pub key: u64,
}

#[derive(Copy, Clone)]
pub struct ResultIndex {
    pub result: u64,
    pub index: i32,
}


impl Cell {
    pub fn to_string(self) -> String {
        return format!("plain {} , key {}", self.plain, self.key);
    }
}

enum Message {
    NewJob(Job),
    Terminate,
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<FnBox + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        //println!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        //println!("Shutting down all workers.");

        for worker in &mut self.workers {
            //println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        //println!("Worker {} got a job; executing.", id);

                        job.call_box();
                    }
                    Message::Terminate => {
                        //println!("Worker {} was told to terminate.", id);

                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}