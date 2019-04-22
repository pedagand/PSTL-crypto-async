use crate::{Cell,Scheduler};
use std::sync::{Arc, Mutex};
use std::{thread, time};

pub fn demo_submit_job(buf: Arc<Mutex<Vec<Cell>>>, scheduler: Arc<Scheduler>, size: usize,
                       lock_plain: Arc<Mutex<u64>>, lock_key: Arc<Mutex<u64>>) {

    let mut cpt = scheduler.counter_index.lock().unwrap();
    if *cpt == -1 {
        if size != 1 {
            scheduler.chan_wait_to_write.lock().unwrap().recv().unwrap();
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
        scheduler.chan_wait_to_encrypt.lock().unwrap().recv().unwrap();
        let mut buff = buf.lock().unwrap();
        for i in 0..(size) {
            buff[i].plain ^= buff[i].key;
            thread::sleep(time::Duration::from_millis(1));
        }
        let result = buff[index as usize].plain;
        std::mem::drop(buff);

        let mut cpt = scheduler.counter_index.lock().unwrap();
        *cpt = -1;
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
        let buff = buf.lock().unwrap();
        let result = buff[index as usize].plain;
        std::mem::drop(buff);

        assert!(result == local_plain ^ local_key);
        let mut c = scheduler.counter_write.lock().unwrap();
        *c += 1;
        if *c == (size as i32) - 1 {
            scheduler.chan_ok_to_write.lock().unwrap().send(()).unwrap();
            *c = 0;
        }
    }
}
