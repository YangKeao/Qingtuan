use crate::memtable::Slice;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::net::TcpStream;
use std::sync::{Arc, RwLock};

pub struct PutOp(Slice, Slice);
pub enum Operation {
    Put(PutOp),
}

pub struct Handle {
    op: Operation,
    stream: Arc<RwLock<TcpStream>>,
}

impl Handle {
    pub fn new(op: Operation, stream: Arc<RwLock<TcpStream>>) -> Handle {
        Handle { op, stream }
    }
}

pub struct Database {
    r: Receiver<Handle>,
    s: Sender<Handle>,
}

impl Database {
    pub fn new() -> Database {
        let (s, r) = unbounded();
        Database { r, s }
    }

    pub fn get_sender(&self) -> Sender<Handle> {
        self.s.clone()
    }
}
