use crate::internal_database::InternalDatabase;
use crate::protocol_parser::Protocol;
use crate::slice::Slice;
use crossbeam::channel::{unbounded, Sender};
use std::sync::{Arc, RwLock};
use std::thread;

pub struct PutOp(pub Slice, pub Slice);
pub struct GetOp(pub Slice);
pub enum Operation {
    Put(PutOp),
    Get(GetOp),
}

pub struct GetReturn(pub Slice);
pub enum Return {
    Get(GetReturn),
}

pub struct Handle {
    pub op: Operation,
    pub reply_sender: Protocol, // TODO: Add Reply Type
}

impl Handle {
    pub fn new(op: Operation, reply_sender: Protocol) -> Handle {
        Handle { op, reply_sender }
    }
}

pub struct Database {
    s: Sender<Handle>,
    internal_database: Arc<RwLock<InternalDatabase>>,
}

impl Database {
    pub fn new() -> Database {
        let (s, r) = unbounded::<Handle>();
        let internal_database = Arc::new(RwLock::new(InternalDatabase::new()));

        let db = internal_database.clone();
        thread::spawn(move || {
            while let Ok(handle) = r.recv() {
                match handle.op {
                    Operation::Get(op) => {
                        db.read().unwrap().get(op.0);
                    }
                    Operation::Put(op) => {
                        db.write().unwrap().put(op.0, op.1);
                    }
                }
            }
        });
        Database {
            s,
            internal_database,
        }
    }

    pub fn get_sender(&self) -> Sender<Handle> {
        self.s.clone()
    }
}
