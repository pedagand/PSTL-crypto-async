extern crate threaded;


#[cfg(test)]
mod tests {
    use rand::prelude::*;
    use std::sync::{Arc, Mutex, Barrier};
    use web_server::{ThreadPool, Scheduler, ResultIndex};

    #[test]
    fn test_scheduler() {
        const nb_request: usize = 64;
        let vec_result: Vec<ResultIndex> = vec![ResultIndex { result: 0, index: 0 }; nb_request];
        let vec_result: Arc<Mutex<Vec<ResultIndex>>> = Arc::new(Mutex::new(vec_result));
        let bench_size = 64;
        let pool = ThreadPool::new(bench_size);
        let size: usize = bench_size;
        let mut plain: [u64; 64] = [0; 64];
        let mut key: [u64; 64] = [0; 64];
        for i in 0..64 {
            plain[i] = rand::thread_rng().gen();
            key[i] = rand::thread_rng().gen();
        }

        let scheduler = Scheduler::new(size);
        let scheduler = Arc::new(scheduler);
        let barrier = Arc::new(Barrier::new(bench_size + 1));

        for i in 0..nb_request {
            let plain = plain[i];
            let key = key[i];
            let k = Arc::new(Mutex::new(key));
            let p = Arc::new(Mutex::new(plain));
            let scheduler = Arc::clone(&scheduler);
            let plain = Arc::clone(&p);
            let key = Arc::clone(&k);
            let c = barrier.clone();
            let vec_result = Arc::clone(&vec_result);

            pool.execute(move || {
                let res = threaded::runtime::submit_job(scheduler, size, key, plain);
                let mut vec_result = vec_result.lock().unwrap();
                vec_result.push(res);
                std::mem::drop(vec_result);
                c.wait();
            });
        }
        barrier.wait();
        let mut buff_result: [u64; nb_request] = [0; 64];
        for res in vec_result.lock().unwrap().iter() {
            buff_result[res.index as usize] = res.result;
        }
        for i in 0..nb_request {
            assert_eq!(buff_result[i], plain[i] ^ key[i]);
        }
    }
}

