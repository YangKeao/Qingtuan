use crate::database::{Database, Handle};
use crate::protocol_parser::Protocol;
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::{Arc, RwLock};
use std::thread;

pub struct Qingtuan {
    listener: TcpListener,
    database: Arc<Database>,
}

impl Qingtuan {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Qingtuan {
        Qingtuan {
            listener: TcpListener::bind(addr).unwrap(),
            database: Arc::new(Database::new()),
        }
    }

    pub fn listen(&mut self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let operation_sender = self.database.get_sender();
                    thread::spawn(move || {
                        let protocol = Protocol::from(stream);

                        for op in protocol.iter() {
                            operation_sender
                                .send(Handle::new(op, protocol.get_sender()))
                                .unwrap();
                        }
                    });
                }
                Err(_) => {}
            }
        }
    }
}
