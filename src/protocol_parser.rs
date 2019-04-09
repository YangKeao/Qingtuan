use crate::database::*;
use crate::memtable::Slice;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Read;
use std::net::TcpStream;

pub struct ProtocolParser {
    stream: TcpStream,
}

impl From<TcpStream> for ProtocolParser {
    fn from(stream: TcpStream) -> Self {
        return ProtocolParser { stream };
    }
}

enum Prefix {
    Count,
    String,
}

trait ProtocolReader {
    fn read_prefix(&mut self) -> Prefix;
    fn read_num(&mut self) -> i32;
    fn read_str(&mut self) -> String;
    fn read_op(&mut self) -> Operation;
    fn read_slice(&mut self) -> Slice;
}

impl ProtocolReader for TcpStream {
    fn read_prefix(&mut self) -> Prefix {
        match self.read_u8().unwrap().into() {
            '*' => Prefix::Count,
            '$' => Prefix::String,
            _ => {
                panic!("Unknown prefix") // TODO: Better Error Handling
            }
        }
    }

    fn read_num(&mut self) -> i32 {
        self.read_i32::<BigEndian>().unwrap()
    }

    fn read_str(&mut self) -> String {
        let prefix = self.read_prefix();
        match prefix {
            Prefix::String => {
                let length = self.read_num();
                let mut buffer = Vec::with_capacity(length as usize);
                self.read_exact(buffer.as_mut_slice()).unwrap();

                String::from_utf8(buffer).unwrap()
            }
            _ => {
                panic!("The start of string should be $") // TODO: Better Error Handling
            }
        }
    }

    fn read_slice(&mut self) -> Slice {
        let prefix = self.read_prefix();
        match prefix {
            Prefix::String => {
                let length = self.read_num();
                let mut buffer = Vec::with_capacity(length as usize);
                self.read_exact(buffer.as_mut_slice()).unwrap();

                Slice::from(buffer)
            }
            _ => {
                panic!("The start of slice should be $") // TODO: Better Error Handling
            }
        }
    }

    fn read_op(&mut self) -> Operation {
        let prefix = self.read_prefix();
        match prefix {
            Prefix::Count => {
                let method = self.read_str();
                match &method[..] {
                    "PUT" => Operation::Put(PutOp(self.read_slice(), self.read_slice())),
                    "GET" => Operation::Get(GetOp(self.read_slice())),
                    _ => panic!("Unkown Method"),
                }
            }
            _ => {
                panic!("Start of an operation should be *") // TODO: Better Error Handling
            }
        }
    }
}

impl Into<(TcpStream, Vec<Operation>)> for ProtocolParser {
    fn into(self) -> (TcpStream, Vec<Operation>) {
        let mut ops = Vec::new();

        (self.stream, ops) // TODO: add parse function
    }
}
