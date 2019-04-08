use crate::database::Operation;
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct ProtocolParser {
    stream: TcpStream,
}

impl From<TcpStream> for ProtocolParser {
    fn from(stream: TcpStream) -> Self {
        return ProtocolParser { stream };
    }
}

impl Into<(TcpStream, Vec<Operation>)> for ProtocolParser {
    fn into(self) -> (TcpStream, Vec<Operation>) {
        (self.stream, Vec::new()) // TODO: add parse function
    }
}
