extern crate crypto;

use std::error::Error;
use std::fmt;
use crypto::aes::{self, KeySize};

#[derive(Clone, Copy)]
enum Status {
    Idle,
    BeingProcessed,
    Completed
}

fn from_char_to_u8(list : &[char], mut list_u8 : &[u8]){
    if list.len() != list_u8.len() {
        panic!("Array of different size");
    }
    let mut vect : Vec<u8> = Vec::with_capacity(list.len());

    for i in list {
        &vect.push(*i as u8);
    }
    //list_u8 = &vect[..];
}

#[derive(Debug, Clone, Copy)]
struct AesJobInLaneError;

impl fmt::Display for AesJobInLaneError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error AEs job in lane")
    }
}

impl Error for AesJobInLaneError {
    fn description(&self) -> &str {
        "No job in lane"
    }
}

#[derive(Clone)]
struct Aes_job{
    plaintext : Box<[u8]>,
    ciphertext : Box<[u8]>,
    //iv : [char;16],
    iv : [u8;16],
    keys : Box<[u8]>,
    len : u8,
    status : Status
}


struct Aes_args{
    //change the char in sometihing else
    input : [Box<[u8]>;8],
    output : [Box<[u8]>;8],
    //iv : [[char;16];8],
    iv : [[u8;16];8],
    keys : [Box<[u8]>;8]
}


struct Manager {
    args : Aes_args,
    lens: [u32;8],
    unused_lanes: Vec<u8>,
    job_in_lane : [Result<Aes_job, AesJobInLaneError>;8]
    //job_in_lane : [Aes_job;8]
}

impl Manager {
    fn submit_job(&mut self, mut job : Aes_job){

        let lane_id = self.unused_lanes.pop();
        match lane_id {
            Some(lane) => {
                self.job_in_lane[lane as usize] = Ok(job.clone());

                self.args.input[lane as usize] = job.plaintext.clone();
                self.args.output[lane as usize] = job.ciphertext.clone();
                self.args.keys[lane as usize] = job.keys.clone();
                self.args.iv[lane as usize] = job.iv.clone();

                job.status = Status::BeingProcessed;
                if self.unused_lanes.len() == 0 {
                    return
                }
                let (minIdx, min) = self.getMin();
                for i in 0..min {
                    let inp : & mut[u8] = Box::leak(self.args.input[i as usize].clone());
                    let mut out : &mut [u8] =  Box::leak(self.args.output[i as usize].clone());
                    let key : &mut [u8] = Box::leak(self.args.keys[i as usize].clone());
                    //let iv_char  : [char;16] = self.args.iv[i as usize];
                    let iv : [u8;16] = self.args.iv[i as usize];
                    ctr_encryption(&inp, &mut out, &key, &iv);
                }

                for i in 0..8 {
                    self.lens[i] = self.lens[i]-min as u32;
                }

                for i in 0..8 {
                    if self.lens[i] == 0{
                        match &self.job_in_lane[i] {
                            Ok(mut J) => {J.status = Status::Completed;
                            self.unused_lanes.push(i as u8);
                            },
                            Err(E) => println!("Cant change the status"),
                        }

                    }
                }
            }
            None    => println!("There is no lane available"),
        }

    }

    fn getMin(&self) -> (u32, u8) {
        let mut index : u8 = 0;
        let mut min_size : u32 = self.lens[0];
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

fn build_Args() -> Aes_args {
    let b_input : Box<[u8]> = Box::new([0;0]);
    let input : [Box<[u8]>; 8] = [b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone()];
    let output : [Box<[u8]>; 8] = [b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone()];
    let keys : [Box<[u8]>; 8] = [b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone()];
    let iv = [[0;16];8];
    Aes_args{
        input : input,
        output : output,
        iv : iv,
        keys : keys
    }
}

fn main() {
    
    let input : [u8;75] = [0;75];
    let output : [u8;75] = [0;75];
    let mut key: [u8;16] = [1;16] ;
    let mut iv : [u8;16] = [2;16] ;;
    let inp = Box::new(input);
    let out = Box::new(output);
    let keys = Box::new(key);
    let job1 = Aes_job{plaintext : inp,
         ciphertext : out,
         iv : iv,
         keys : keys,
         len : input.len() as u8,
         status : Status::Idle};
    let aes_args : Aes_args = build_Args();
    let job_err = Err(AesJobInLaneError);
    let job_n_lane: [Result<Aes_job,AesJobInLaneError>;8] = [job_err.clone(),job_err.clone(),job_err.clone(),job_err.clone(),job_err.clone(),job_err.clone(),job_err.clone(),job_err.clone()];
    let manager = Manager{args : aes_args,
        lens : [0;8],
        unused_lanes : Vec::new(),
        job_in_lane : job_n_lane};


}

/*
let test : [char; 8] = ['a'; 8];
let test_u8 : [u8;8] = [0;8];
//from_char_to_u8(&test, &test_u8);

println!("{:?}", test_u8);
*/
