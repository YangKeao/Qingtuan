use crate::database::{Database, Handle};
use crate::protocol_parser::ProtocolParser;
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::{Arc, RwLock};
use std::thread;

pub struct Engine {
    listener: TcpListener,
    database: Arc<Database>,
}

impl Engine {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Engine {
        Engine {
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
                        let protocol_parser = ProtocolParser::from(stream);
                        let (stream, ops) = protocol_parser.into();
                        let stream = Arc::new(RwLock::new(stream));

                        for op in ops {
                            operation_sender
                                .send(Handle::new(op, stream.clone()))
                                .unwrap();
                        }
                    });
                }
                Err(_) => {}
            }
        }
    }
}
