use bytes::BufMut;

use crate::transport::pool::Kind;
use crate::transport::constants::{
    FRAME_LEN_BYTES, FRAME_KIND_BYTES, FRAME_HEADER_BYTES
};

#[derive(Debug)]
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

    pub fn from_slice(slice: &[u8]) -> Self {
        let kind = slice[FRAME_LEN_BYTES];

        // Frame data
        let data = Vec::with_capacity(slice.len()-FRAME_HEADER_BYTES);
        data.copy_from_slice(&slice[FRAME_HEADER_BYTES..]);

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
    start_data: usize,
    end_data: usize
    part_frame: Option<(usize, Frame)>
}

impl Buffer {
    pub fn new(buffer_size: usize) -> Buffer {
        Buffer {
            content: vec![0; buffer_size],
            start_data: 0,
            end_data: 0,
            part_frame: None,
        }
    }

    pub fn get_slice_to_read(&mut self) -> &mut [u8] {
        &mut self.content[self.end_data..]
    }

    pub fn readed_bytes(&mut self, n: usize) {
        self.end_data += n;
    }

    fn current_byte(&mut self) -> u8 {
        self.start_data += 1;
        self.content[self.start_data - 1]
    }

    fn get_header(&mut self) -> Option<(usize, usize)> {
        if self.end_data - self.start_pointer >= FRAME_LEN_BYTES + FRAME_KIND_BYTES {
            todo!()
        } else {
            for (new_index, old_index) in (self.start_data..self.end_data).enumerate() {
                self.content[new_index] = self.content[old_index];     
            }
            self.end_data = self.data_len();
            self.start_data = 0;
            None
        }
    }

    pub fn try_get_frame(&mut self) -> Option<Frame> {
        if let Some((len, part_frame)) = self.part_frame.move() {
            if len <= part_frame.len() + self.data_len() {
                while part_frame.len() < len() {
                    part_frame.data.push(self.current_byte())
                }
            } else {
                self.part_frame = Some((len, part_frame));
                None
            }
        } else {
            let header = self.get_header()?;
            if self.data_len() >= header {
                let mut result = Frame::default();
                let start = self.data_interval.0;
                let end = start + header + 1;
                self.data_interval.0 += header;
                Some(Frame::from_slice(self.connect[start..end]))
            } else {
                todo!()
            }
        }
    }

    pub fn empty(&self) -> bool {
        self.start_data == self.end_data
    }

    fn data_len(&self) -> usize {
        self.end_data - self.start_data
    }

    pub fn try_get_part_frame(&mut self) -> Option<PartFrame> {
        let header = self.get_header()?;

        if !self.empty() && self.data_len() < header {

            let start = self.data_interval.0;
            let end = self.data_interval.1;

            Some(PartFrame::new(self.content[start..end]))
        } else {
            self.data_interval = backup;
            None
        }
    }
}