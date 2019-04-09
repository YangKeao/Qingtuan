use crate::database::*;
use crate::slice::Slice;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Read;
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

// trait ProtocolReader {
//     fn read_prefix(&mut self) -> Prefix;
//     fn read_num(&mut self) -> i32;
//     fn read_str(&mut self) -> String;
//     fn read_op(&mut self) -> Operation;
//     fn read_slice(&mut self) -> Slice;
// }

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
        {
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
            }
        }
    }
}
