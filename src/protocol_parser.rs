use crate::database::*;
use crate::slice::Slice;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct Protocol {
    stream: TcpStream,
}

impl From<TcpStream> for Protocol {
    fn from(stream: TcpStream) -> Self {
        return Protocol { stream };
    }
}

pub enum Prefix {
    Count,
    String,
}

pub trait ProtocolReader: Read {
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

    fn read_vec(&mut self) -> Vec<u8> {
        let prefix = self.read_prefix();
        match prefix {
            Prefix::String => {
                let length = self.read_num();
                self.read_break_line();
                let mut buffer = vec![0; length as usize];
                self.read_exact(buffer.as_mut_slice()).unwrap();
                self.read_break_line();
                return buffer;
            }
            _ => {
                panic!("The start of string should be $") // TODO: Better Error Handling
            }
        }
    }

    fn read_str(&mut self) -> String {
        String::from_utf8(self.read_vec()).unwrap()
    }

    fn read_break_line(&mut self) {
        let mut buffer = vec![0; 2];
        self.read_exact(buffer.as_mut_slice()).unwrap();

        if buffer[0] == '\r' as u8 && buffer[1] == '\n' as u8 {
            return;
        } else {
            panic!("Expect a break line here");
        }
    }

    fn read_slice(&mut self) -> Slice {
        Slice::from(self.read_vec())
    }

    fn read_op(&mut self) -> Operation {
        let prefix = self.read_prefix();
        self.read_break_line();
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

impl<T: Read> ProtocolReader for T {}

pub trait ProtocolWriter: Write {
    fn write_prefix(&mut self, prefix: Prefix) {
        match prefix {
            Prefix::Count => self.write_u8('*' as u8).unwrap(),
            Prefix::String => self.write_u8('$' as u8).unwrap(),
        };
    }
    fn write_num(&mut self, num: i32) {
        self.write_i32::<BigEndian>(num).unwrap();
    }
    fn write_break_line(&mut self) {
        self.write(b"\r\n").unwrap();
    }
    fn write_vec(&mut self, vec: Vec<u8>) {
        self.write_prefix(Prefix::String);
        self.write_num(vec.len() as i32);
        self.write_break_line();
        self.write(vec.as_slice()).unwrap();
        self.write_break_line();
    }
    fn write_str(&mut self, str: String) {
        self.write_vec(str.into_bytes());
    }
    fn write_slice(&mut self, slice: Slice) {
        self.write_vec(slice.into());
    }
    fn write_return(&mut self, ret: Return) {
        match ret {
            Return::Get(get_return) => self.write_slice(get_return.0),
        }
    }
}

impl<T: Write> ProtocolWriter for T {}

impl Iterator for Protocol {
    type Item = Operation;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.stream.read_op())
    }
}

impl Clone for Protocol {
    fn clone(&self) -> Protocol {
        Protocol {
            stream: self.stream.try_clone().unwrap(),
        }
    }
}

impl Protocol {
    pub fn iter(&self) -> Protocol {
        self.clone()
    }

    pub fn get_sender(&self) -> Protocol {
        self.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::slice::Slice;

    #[test]
    fn parse_operation() {
        let buffer = vec![
            '*' as u8, '\r' as u8, '\n' as u8, '$' as u8, 0, 0, 0, 3, '\r' as u8, '\n' as u8,
            'G' as u8, 'E' as u8, 'T' as u8, '\r' as u8, '\n' as u8, '$' as u8, 0, 0, 0, 1,
            '\r' as u8, '\n' as u8, 'Y' as u8, '\r' as u8, '\n' as u8,
        ];
        let mut buffer = &buffer[..];
        let op = buffer.read_op();
        match op {
            Operation::Get(op) => assert!(op.0 == Slice::from(vec!['Y' as u8])),
            _ => panic!("Parse Operation Error"),
        };
    }

    #[test]
    fn parse_return() {
        let mut buffer = Vec::new();

        buffer.write_return(Return::Get(GetReturn(Slice::from(String::from("GET")))));

        assert_eq!(
            buffer,
            vec![
                '$' as u8, 0, 0, 0, 3, '\r' as u8, '\n' as u8, 'G' as u8, 'E' as u8, 'T' as u8,
                '\r' as u8, '\n' as u8
            ]
        )
    }
}
