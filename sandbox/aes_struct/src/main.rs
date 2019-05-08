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

fn from_char_to_u8(list_char : &Vec<char>, list_u8 : &mut Vec<u8>){
    for i in list_char {
        &list_u8.push(*i as u8);
    }
}

fn from_u8_to_char(list_char : &mut Vec<char>, list_u8 : &Vec<u8>){
    let mut vec : Vec<char> = Vec::with_capacity(list_u8.len());
    for i in 0..list_u8.len() {
        list_char[i] = (i as u8) as char;
    }
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
    plaintext : Vec<char>,
    ciphertext : Vec<char>,
    //iv : [char;16],
    iv : [u8;16],
    keys : Vec<char>,
    len : u32,
    status : Status
}


struct Aes_args{
    //change the char in sometihing else
    input : [Vec<char>;8],
    output : [Vec<char>;8],
    //iv : [[char;16];8],
    iv : [[u8;16];8],
    keys : [Vec<char>;8]
}


struct Manager {
    args : Aes_args,
    lens: [u32;8],
    unused_lanes: Vec<u8>,
    job_in_lane : [Result<Aes_job, AesJobInLaneError>;8]
}

impl Manager {
    fn submit_job(&mut self, mut job :  Aes_job){

        let lane_id = self.unused_lanes.pop();
        match lane_id {
            Some(lane) => {
                self.job_in_lane[lane as usize] = Ok(job.clone());

                self.args.input[lane as usize] = job.plaintext.clone();
                self.args.output[lane as usize] = job.ciphertext.clone();
                self.args.keys[lane as usize] = job.keys.clone();
                self.args.iv[lane as usize] = job.iv.clone();

                job.status = Status::BeingProcessed;
                self.lens[lane as usize] = job.len ;
                if self.unused_lanes.len() != 0 {
                    return
                }

                let (minIdx, min) = self.getMin();
                for i in 0..8 {
                    let mut inp : Vec<u8> = Vec::with_capacity(self.args.input[i as usize].len());
                    from_char_to_u8(&self.args.input[i as usize].clone(), &mut inp);
                    let mut out :Vec<u8> = Vec::with_capacity(self.args.output[i as usize].len());
                    from_char_to_u8(&self.args.output[i as usize].clone(), &mut out);
                    let mut key : Vec<u8> = Vec::with_capacity(self.args.keys[i as usize].len());
                    from_char_to_u8(&self.args.keys[i as usize].clone(), &mut key);
                    //let iv_char  : [char;16] = self.args.iv[i as usize];
                    let iv : [u8;16] = self.args.iv[i as usize];
                    ctr_encryption(&inp, &mut out, &key, &iv);
                    from_u8_to_char(&mut self.args.output[i as usize], &out);
                    println!("out : {:?}", out);
                }

                for i in 0..8 {
                    self.lens[i] = self.lens[i]-min as u32;
                }

                for i in 0..8 {
                    if self.lens[i] == 0{
                        match &self.job_in_lane[i] {
                            Ok(j) => {
                                //j.status = Status::Completed;
                                self.unused_lanes.push(i as u8);
                                self.job_in_lane[i] = Err(AesJobInLaneError);
                            },
                            Err(e) => println!("Error : {}", e),
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


    fn change_status(job : &mut Aes_job, status : Status){
        job.status = status;
    }
}

fn ctr_encryption(input : &[u8], mut output : &mut [u8], key : &[u8], nonce : &[u8]) {
    //initialize the Encryptor
    let mut _cipher = aes::ctr(KeySize::KeySize128, key, nonce);
    _cipher.process(input, &mut output);
}

fn build_Args() -> Aes_args {
    let b_input : Vec<char> = Vec::new();
    let input : [Vec<char>; 8] = [b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(),
    b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone()];
    let output : [Vec<char>; 8]  = [b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(),
    b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone()];
    let keys : [Vec<char>; 8]  = [b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone(), b_input.clone()];
    let iv = [[0;16];8];
    Aes_args{
        input : input,
        output : output,
        iv : iv,
        keys : keys
    }
}


fn build_Manager(args : Aes_args) -> Manager {
    let job_in_lane : [Result<Aes_job, AesJobInLaneError>; 8] = [Err(AesJobInLaneError),Err(AesJobInLaneError),Err(AesJobInLaneError),Err(AesJobInLaneError),
                                                                Err(AesJobInLaneError),Err(AesJobInLaneError),Err(AesJobInLaneError),Err(AesJobInLaneError)];
    let mut unused : Vec<u8> = Vec::new();
    unused.push(7);unused.push(6);unused.push(5);unused.push(4);
    unused.push(3);unused.push(2);unused.push(1);unused.push(0);
    Manager {
        args : args,
        lens : [0;8],
        unused_lanes : unused,
        job_in_lane : job_in_lane
    }
}

fn main() {

    let mut args = build_Args();
    let mut manager = build_Manager(args);

    let mut input : Vec<char> = Vec::new();
        input.push('a');input.push('a');input.push('a');input.push('a');input.push('a');
        input.push('a');input.push('a');input.push('a');input.push('a');input.push('a');
        input.push('a');input.push('a');input.push('a');input.push('a');

        let mut output : Vec<char> = input.clone();
        let mut keys : Vec<char> = input.clone();
        keys.push('a');keys.push('a');
        let len : u32 = input.len() as u32;

        let mut job : Aes_job = Aes_job {
            plaintext : input.clone(),
            ciphertext : output,
            iv : [0;16],
            len : len,
            keys : keys.clone(),
            status : Status::Idle
        };

        let mut job2 = job.clone();
        keys.pop();
        keys.push('b');
        job2.keys =  keys.clone();
        let mut job3 = job.clone();
        keys.pop();
        keys.push('c');
        job3.keys =  keys.clone();
        let mut job4 = job.clone();
        keys.pop();
        keys.push('d');
        job4.keys =  keys.clone();
        let mut job5 = job.clone();
        keys.pop();
        keys.push('e');
        job5.keys =  keys.clone();
        let mut job6 = job.clone();
        keys.pop();
        keys.push('f');
        job6.keys =  keys.clone();
        let mut job7 = job.clone();
        keys.pop();
        keys.push('g');
        job7.keys =  keys.clone();
        let mut job8 = job.clone();
        keys.pop();
        keys.push('h');
        input.push('b');
        job8.plaintext = input.clone();
        job8.ciphertext = input.clone();
        job8.keys =  keys.clone();

        manager.submit_job(job);
        manager.submit_job(job2);
        manager.submit_job(job3);
        manager.submit_job(job4);
        manager.submit_job(job5);
        manager.submit_job(job6);
        manager.submit_job(job7);
        manager.submit_job(job8);

        /*println!("job1 : {:?}", job.ciphertext);
        println!("job2 : {:?}", job2.ciphertext);
        println!("job3 : {:?}", job3.ciphertext);
        println!("job4 : {:?}", job4.ciphertext);
        println!("job5 : {:?}", job5.ciphertext);
        println!("job6 : {:?}", job6.ciphertext);
        println!("job7 : {:?}", job7.ciphertext);
        println!("job8 : {:?}", job8.ciphertext);*/

}
