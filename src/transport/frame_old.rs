pub struct PartFrame {
    len: usize,
    data: Vec<u8>,
}

impl PartFrame {
    pub fn new(len: usize, data: &[u8]) -> PartFrame {
        let mut part_frame_data = Vec::with_capacity(len);

        for i in 0..data.len() {
            part_frame_data[i] = data[i];
        }

        PartFrame {
            len,
            data: part_frame_data,
        }
    }
}

#[derive(Default, Debug)]
pub struct Frame {
    pub data: Vec<u8>,
}

impl Frame {
    pub fn new(data: &[u8]) -> Frame {
        let mut frame_data = Vec::with_capacity(data.len());

        for i in 0..data.len() {
            frame_data[i] = data[i];
        }

        Frame {
            data: frame_data,
        }
    }

    pub fn push(&mut self, val: u8) {
        self.data.push(val);
    }
}