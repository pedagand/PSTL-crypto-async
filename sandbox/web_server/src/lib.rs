use std::sync::{Arc, mpsc, Mutex, RwLock};
use std::{thread, time};


pub fn submit_job(scheduler: Arc<Scheduler>, size: usize,
                  lock_plain: Arc<Mutex<u64>>, lock_key: Arc<Mutex<u64>>) -> ResultIndex {
    let mut cpt = scheduler.counter_index.lock().unwrap();

    if *cpt == -1 {
        ///  Ce premier wait empêche d'autres threads d'écrire dans le buffer
    /// tant que les premières tâches n'ont pas fini de lire le résultat calculé par la
    /// dernière thread. L'attente est donc terminée à la ligne 67, lorsque la variable
    /// counter_write, qui sert normalement de compteur pour le nombre de résultat envoyé
    /// au client, atteint size - 1.
        scheduler.chan_wait_to_write.lock().unwrap().recv().unwrap();
        *cpt = 0;
    }
    let index = *cpt;
    *cpt += 1;
    std::mem::drop(cpt);
    assert!(index >= 0 && index <= size as i32);
    let mut buff = scheduler.buffer.lock().unwrap();
    let plain = lock_plain.lock().unwrap();
    let key = lock_key.lock().unwrap();
    buff[index as usize].key = *key;
    buff[index as usize].plain = *plain;
    let local_plain = *plain;
    let local_key = *key;

    std::mem::drop(buff);
    std::mem::drop(plain);
    std::mem::drop(key);

    if index == size as i32 - 1 || size == 1 {
        //Le deuxième wait, met en attente la dernière thread qui est libéré par le send du
        //Sender chan_ok_to_read à la ligne 58. Le send est envoyé seulement lorsque la variable
        // counter_wait, qui est incrémenté après qu'une thread a écrit dans le buffer son plain
        // et key atteint size -1. Ce wait permet donc d'attendre que toute les threads terminent
        // d'écrire dans le buffer avant que le dernier commence le calcul.
        if size !=1 {
            scheduler.chan_wait_to_encrypt.lock().unwrap().recv().unwrap();
        }
        let mut buff = scheduler.buffer.lock().unwrap();
        let mut crypt_buffer = scheduler.crypt_buff.write().unwrap();
        for i in 0..(size) {
            crypt_buffer[i] = buff[i].plain ^ buff[i].key;
            thread::sleep(time::Duration::from_millis(1));
        }
        let result = crypt_buffer[index as usize];
        std::mem::drop(buff);
        std::mem::drop(crypt_buffer);
        let mut cpt = scheduler.counter_index.lock().unwrap();

        if size != 1 {
            *cpt = -1;
        } else { *cpt = 0 }
        std::mem::drop(cpt);
        for _ in 0..size - 1 {
            ///liberation du troisieme wait
            scheduler.chan_ok_to_read.lock().unwrap().send(()).unwrap();
        }
        return ResultIndex { result, index };
    } else {
        let mut c_wait = scheduler.counter_wait.lock().unwrap();
        *c_wait += 1;
        if *c_wait == (size as i32) - 1 {
            ///liberation du deuxieme wait
            scheduler.chan_ok_to_encrypt.lock().unwrap().send(()).unwrap();
            *c_wait = 0;
        }
        std::mem::drop(c_wait);
        ///Le troisième wait est terminé par le send de chan_ok_to_read est réalisé après que la
        /// dernière thread finisse de faire le calcul. Ce point de synchronisation  met donc en
        /// attente les thread pendant le temps du calcul.
        scheduler.chan_wait_to_read.lock().unwrap().recv().unwrap();
        let mut crypt_buffer = scheduler.crypt_buff.read().unwrap();
        let result = crypt_buffer[index as usize];
        assert!(result == local_plain ^ local_key);
        let mut c = scheduler.counter_write.lock().unwrap();
        *c += 1;
        if *c == (size as i32) - 1 {
            ///liberation du premier wait
            scheduler.chan_ok_to_write.lock().unwrap().send(()).unwrap();
            *c = 0;
        }
        return ResultIndex { result, index };
    }
}


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
    pub crypt_buff: Arc<RwLock<Vec<u64>>>,
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
        let crypt_buff: Arc<RwLock<Vec<u64>>> = Arc::new(RwLock::new(vec![0; size]));

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