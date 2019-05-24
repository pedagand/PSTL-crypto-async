extern crate crypto;
extern crate rand;

use std::iter::repeat;
use rand::{OsRng, Rng};
use std::sync::Mutex;
use std::error::Error;
use std::fmt;
use std::cmp;
use std::sync::{Arc, RwLock};
//use crypto::aes::{self, KeySize};

#[derive(PartialEq, Debug)]
enum Status {
    BeingProcessed,
    Completed
}


struct Job{
    plaintext: Vec<u8>,
    iv: [u8;16],
    keys: Vec<u8>,
    len: usize
}

struct Receipt {
    ciphertext: Vec<u8>,
    status: Status,
}

struct Manager {
    jobs: Vec<Job>,
    min_len: usize,
    receipts: Vec<Arc<RwLock<Receipt>>>,
}

impl Manager {
    fn new () -> Manager {
        return Manager {
            jobs: Vec::new(),
            min_len: usize::max_value(),
            receipts: Vec::new()
        }
    }

    fn submit_job(&mut self, job: Job) -> Arc<RwLock<Receipt>> {

        let mut submitted = Receipt { ciphertext: Vec::new(),
                                  status: Status::BeingProcessed };

        submitted.ciphertext = vec![0; job.len];

        let p = Arc::new(RwLock::new(submitted));

        self.receipts.push(p.clone());
        self.min_len = cmp::min(self.min_len, job.len);
        self.jobs.push(job);

        //println!("min len : {:?}", &self.min_len);
        if self.jobs.len() == 8 {
            // Batch encryption (faked)
            for (i, job) in self.jobs.iter().enumerate() { // XXX: get rid of clone() here

                fake_encrypt(&job.plaintext,
                             &mut Arc::clone(&self.receipts[i]).write().unwrap().ciphertext,
                             &job.keys,
                             &job.iv,
                             self.min_len);
            }

            for i in 0..self.jobs.len() {
                self.jobs[i].len -= self.min_len;
                if self.jobs[i].len == 0 {
                    Arc::clone(&self.receipts[i]).write().unwrap().status = Status::Completed;
                }
            }

            //Remove finished job
            self.jobs.retain(|x| x.len != 0);
            //Remove finished receipt
            self.receipts.retain(|x| x.read().unwrap().status == Status::BeingProcessed);

            self.min_len = self.min_len();

        }

        if self.jobs.len() == 0 {
            self.min_len = usize::max_value();
        }

        return p;

    }

    fn max_len(&self) -> usize{
        let mut max : usize = 0;
        for job in &self.jobs{
            if job.len > max {
                max = job.len;
            }
        }
        max
    }

    fn min_len(&self) -> usize{
        let mut min : usize = usize::max_value();
        for job in &self.jobs{
            if job.len < min {
                min = job.len;
            }
        }
        min
    }

    fn flush_job(&mut self) {
        let max : usize = self.max_len();
        
        for job in &mut self.jobs {
            job.plaintext.resize(max, 0 as u8);
            job.keys.resize(max, 0 as u8);
        }

        for rec in &self.receipts {
            Arc::clone(&rec).write().unwrap().ciphertext.resize(max,0);
        }

        for (i, job) in self.jobs.iter().enumerate() {
            fake_encrypt(&job.plaintext,
                         &mut Arc::clone(&self.receipts[i]).write().unwrap().ciphertext,
                         &job.keys,
                         &job.iv,
                         max);
        Arc::clone(&self.receipts[i]).write().unwrap().ciphertext.resize(job.len,0);
        Arc::clone(&self.receipts[i]).write().unwrap().status = Status::Completed;
        }

        self.jobs = Vec::new();
        self.receipts = Vec::new();
        self.min_len = usize::max_value();
    }

}

fn fake_encrypt(input: &[u8], output: &mut [u8], key: &[u8], nonce: &[u8], len: usize) {

    for i in 0..len {
      output[i] = input[i] ^ key[i];
    }
}

fn poll(receipt : Arc<RwLock<Receipt>>) -> Vec<u8>{
    let rec = Arc::clone(&receipt);
    let mut lock : bool = false;

    while lock != true {
        let r = rec.read().unwrap();
        if r.status == Status::Completed {
            lock = !lock;
        }
        else  {
            drop(r);
        }
    }
    let reception = rec.read().unwrap();
    return reception.ciphertext.to_vec()
}

fn main() {
    let mut _manager = Manager::new();

    let mut random = OsRng::new().expect("Failed to get OS random generator");
    let mut rng = rand::thread_rng();
    let mut rec_vec : Vec<Arc<RwLock<Receipt>>> = Vec::new();

    for i in 0..50 {
        if (i % 10) == 0 {
            _manager.flush_job();
        }
        let mut len : usize = rng.gen_range(0, 100) as usize;
        let mut input: Vec<u8> = repeat(0u8).take(len).collect();
        random.fill_bytes(&mut input[..]);
        let iv : [u8;16] = [0;16];
        let mut key: Vec<u8> = repeat(0u8).take(len).collect();
        random.fill_bytes(&mut key[..]);


        let job = Job{
            plaintext : input,
            iv : iv,
            keys : key,
            len : len
        };
        rec_vec.push(_manager.submit_job(job));
    }
    _manager.flush_job();

    for rec in &rec_vec {
        assert_eq!(rec.read().unwrap().status, Status::Completed);
    }
    assert_eq!(rec_vec.len(), 50);

}
