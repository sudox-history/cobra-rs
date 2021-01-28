use bytes::{Buf, BufMut};

use crate::transport::pool::Kind;
use crate::transport::constants::{
    FRAME_LEN_BYTES, FRAME_KIND_BYTES, FRAME_HEADER_BYTES
};

#[derive(Debug, Default)]
pub struct Frame {
    kind: u8,
    pub data: Vec<u8>,
}

impl Frame {
    pub fn create(kind: u8, data: Vec<u8>) -> Self {
        Frame {
            kind,
            data
        }
    }

    pub fn from_slice(kind: u8, slice: &[u8]) -> Self {
        let mut data = Vec::with_capacity(slice.len());
        data.copy_from_slice(slice);

        Frame {
            kind,
            data,
        }
    }

    pub fn len(&self) -> usize {
        FRAME_LEN_BYTES + FRAME_KIND_BYTES + self.data.len()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(self.len());

        // Frame header
        vec.put_uint(self.len() as u64, FRAME_LEN_BYTES);
        vec.put_uint(self.kind  as u64, FRAME_KIND_BYTES);
        
        // Frame data
        vec.copy_from_slice(&self.data);
        
        vec
    }
}

impl Kind<u8> for Frame {
    fn kind(&self) -> u8 {
        self.kind
    }
}

//...[start_pointer...end_pointer)...
// frame::Buff
pub struct Buffer {
    content: Vec<u8>,
    start_pointer: usize,
    end_pointer: usize,
    part_frame: Option<(usize, Frame)>
}

impl Buffer {
    pub fn new(buffer_size: usize) -> Buffer {
        Buffer {
            content: vec![0; buffer_size],
            start_pointer: 0,
            end_pointer: 0,
            part_frame: None,
        }
    }

    pub fn get_slice_to_read(&mut self) -> &mut [u8] {
        &mut self.content[self.end_pointer..]
    }

    pub fn readed_bytes(&mut self, n: usize) {
        self.end_pointer += n;
    }

    fn current_byte(&mut self) -> u8 {
        self.start_pointer += 1;
        self.content[self.start_pointer - 1]
    }

    fn get_header(&mut self) -> Option<(usize, u8)> {
        if self.end_pointer - self.start_pointer >= FRAME_HEADER_BYTES {
            let start = self.start_pointer;
            let end = start + FRAME_LEN_BYTES + 1;
            self.start_pointer += FRAME_HEADER_BYTES;
            let mut buf = &self.content[start..end];
            Some((buf.get_uint(FRAME_LEN_BYTES) as usize, self.content[end]))
        } else {
            for (new_index, old_index) in (self.start_pointer..self.end_pointer).enumerate() {
                self.content[new_index] = self.content[old_index];     
            }
            self.end_pointer = self.data_len();
            self.start_pointer = 0;
            None
        }
    }

    pub fn try_get_frame(&mut self) -> Option<Frame> {
        if let Some((len, mut part_frame)) = self.part_frame.take() {
            if len <= part_frame.len() + self.data_len() {
                while part_frame.len() < len {
                    part_frame.data.push(self.current_byte());
                }
                Some(part_frame)
            } else {
                self.part_frame = Some((len, part_frame));
                None
            }
        } else {
            let (len, kind) = self.get_header()?;
            if self.data_len() >= len {
                let start = self.start_pointer;
                let end = start + len + 1;
                let slice = &self.content[start..end];
                self.start_pointer += len;
                Some(Frame::from_slice(kind, slice))
            } else {
                let start = self.start_pointer;
                let end = self.end_pointer;
                let slice = &self.content[start..end]; 
                self.start_pointer = 0;
                self.end_pointer = 0;
                self.part_frame = Some((len, Frame::from_slice(kind, slice)));
                None
            }
        }
    }

    pub fn empty(&self) -> bool {
        self.start_pointer == self.end_pointer
    }

    fn data_len(&self) -> usize {
        self.end_pointer - self.start_pointer
    }
}
