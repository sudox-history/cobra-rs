use crate::transport::frame::{Frame, PartFrame};

//...[start_pointer...end_pointer)...
pub struct FrameBuffer {
    content: Vec<u8>,
    start_pointer: usize,
    end_pointer: usize,
    header_size: usize,
}

impl FrameBuffer {
    pub fn new(buffer_size: usize, header_size: usize) -> FrameBuffer {
        FrameBuffer {
            content: vec![0; buffer_size],
            start_pointer: 0,
            end_pointer: 0,
            header_size,
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

    fn get_header(&mut self) -> Option<usize> {
        if self.end_pointer - self.start_pointer >= self.header_size {
            let mut result: usize = 0;

            // Intel (x86) -> LE
            // ARM -> BE

            for i in 0..self.header_size {
                result <<= 8;
                result += self.current_byte() as usize;
            }

            Some(result)
        } else {
            None
        }
    }

    fn make_backup(&self) -> (usize, usize) {
        (self.start_pointer, self.end_pointer)
    }

    fn use_backup(&mut self, backup: (usize, usize)) {
        self.start_pointer = backup.0;
        self.end_pointer = backup.1;
    }

    pub fn try_get_frame(&mut self) -> Option<Frame> {
        let backup = self.make_backup();
        let header = self.get_header()?;

        if self.end_pointer - self.start_pointer >= header {
            let mut result = Frame::default();

            let start = self.start_pointer;
            let end = start + header + 1;

            self.start_pointer += header;

            Some(Frame::new(self.connect[start..end]))
        } else {
            self.use_backup(backup);
            None
        }
    }

    pub fn empty(&self) -> bool {
        self.start_pointer < self.end_pointer
    }

    pub fn try_get_part_frame(&mut self) -> Option<PartFrame> {
        let backup = self.make_backup();
        let header = self.get_header()?;

        if !self.empty() && self.end_pointer - self.start_pointer < header {

            let start = self.start_pointer;
            let end = self.end_pointer;

            Some(PartFrame::new(self.content[start..end]))
        } else {
            self.use_backup(backup);
            None
        }
    }
}
