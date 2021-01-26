use crate::transport::pool::Kind;

#[derive(Debug)]
pub struct Frame {
    kind: u8,
    data: Vec<u8>,
}

impl Frame {
    pub fn create(kind: u8, data: Vec<u8>) -> Self {
        Frame {
            kind,
            data
        }
    }
}

impl Kind<u8> for Frame {
    fn kind(&self) -> u8 {
        self.kind
    }
}

