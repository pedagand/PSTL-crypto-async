extern crate crypto;

use std::error::Error;
use std::fmt;
use crypto::aessafe::{AesSafe128Encryptor, AesSafe128Decryptor, AesSafe128EncryptorX8, AesSafe128DecryptorX8};
use crypto::aes::{self, KeySize};



enum Status {
    Being_processed,
    Completed
}

#[derive(Debug)]
struct Aes_job_in_laneError;

impl fmt::Display for Aes_job_in_laneError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error AEs job in lane")
    }
}

impl Error for Aes_job_in_laneError {
    fn description(&self) -> &str {
        "Aes_job_in_lane caused an error"
    }
}

struct Aes_job{
    plaintext : Box<[char]>,
    ciphertext : Box<[char]>,
    iv : [[char;8];16],
    keys : [char;8],
    len : u8,
    status : Status
}


struct Aes_args{
    //change the char in sometihing else
    input : [char;8],
    output : [char;8],
    iv : [[char;8];16],
    keys : [char;8]
}

struct Manager {
    args : Aes_args,
    lens: [u32;8],
    unused_lanes: Vec<u8>,
    job_in_lane : [Result<Aes_job, Aes_job_in_laneError>;8]
}

impl Manager {
    fn submit_job(&self, &mut job : Aes_job){
        let lane = self.unused_lanes.pop();
        self.job_in_lane[lane] = job;

        self.args.input[lane] = job.plaintext;
        self.args.output[lane] = job.ciphertext;
        self.args.keys = job.keys;
        self.args.iv = job.iv;

        job.status = Status::Being_processed;
        if self.unused_lanes.len() == 0 {
            return
        }
        let (minIdx, min) = self.getMin();
        for i in 0..min {
            ctr_encryption(&self.args.input[i], &mut self.args.output[i], &self.args.keys, &self.args.iv);
        }

        for i in 0..8 {
            self.lens[i] = self.lens[i]-min;
        }

        for i in 0..8 {
            if self.lens[i] == 0{
                self.job_in_lane[i].status = Status::Completed;
                self.job_in_lane[i] = null;
                self.unused_lanes.push(i);
            }
        }
    }

    fn getMin(&self) -> (u32, u8) {
        let mut index : u8 = 0;
        let mut min_size : u32 = self.lens[8];
        for i in 0..8 {
            if min_size > self.lens[i] {
                index = i as u8;
                min_size = self.lens[i];
            }
        }

        return (min_size, index)
    }
}

fn ctr_encryption(input : &[u8], mut output : &mut [u8], key : &[u8], nonce : &[u8]) {
    //initialize the Encryptor
    let mut _cipher = aes::ctr(KeySize::KeySize128, key, nonce);
    _cipher.process(input, &mut output);
}
fn main() {

}
