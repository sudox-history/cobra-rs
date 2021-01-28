#[derive(Default, Debug)]
pub struct Frame {
    len: Option<usize>,
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