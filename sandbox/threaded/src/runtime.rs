use crate::{Cell,Scheduler};
use std::sync::{Arc, Mutex};
use std::{thread, time};



///Cette partie s'occupe de l'algorithme du scheduler en faisant abstraction de l'architecture
/// client/serveur, les keys et plains sont donc générés auparavant.

pub fn demo_submit_job(buf: Arc<Mutex<Vec<Cell>>>, scheduler: Arc<Scheduler>, size: usize,
                       lock_plain: Arc<Mutex<u64>>, lock_key: Arc<Mutex<u64>>) {

    let mut cpt = scheduler.counter_index.lock().unwrap();
    if *cpt == -1 {
        if size != 1 {
            ///  Ce premier wait empêche d'autres threads d'écrire dans le buffer
            /// tant que les premières tâches n'ont pas fini de lire le résultat calculé par la
            /// dernière thread. L'attente est donc terminée à la ligne 67, lorsque la variable
            /// counter_write, qui sert normalement de compteur pour le nombre de résultat envoyé
            /// au client, atteint size - 1.
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

        ///Le deuxième wait, met en attente la dernière thread qui est libéré par le send du
        ///Sender chan_ok_to_read à la ligne 58. Le send est envoyé seulement lorsque la variable
        /// counter_wait, qui est incrémenté après qu'une thread a écrit dans le buffer son plain
        /// et key atteint size -1. Ce wait permet donc d'attendre que toute les threads terminent
        /// d'écrire dans le buffer avant que le dernier commence le calcul.
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
            ///liberation du troisieme wait
            scheduler.chan_ok_to_read.lock().unwrap().send(()).unwrap();
        }
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
        let buff = buf.lock().unwrap();
        let result = buff[index as usize].plain;
        std::mem::drop(buff);

        assert!(result == local_plain ^ local_key);
        let mut c = scheduler.counter_write.lock().unwrap();
        *c += 1;
        if *c == (size as i32) - 1 {
            ///liberation du premier wait
            scheduler.chan_ok_to_write.lock().unwrap().send(()).unwrap();
            *c = 0;
        }
    }
}
