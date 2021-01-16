pub struct FrameBuffer {
    content: Vec<u8>,
    front_pointer: usize,
    max_read_size: usize,
}

impl FrameBuffer {
    pub fn new(buffer_size: usize, max_read_size: usize) -> FrameBuffer {
        FrameBuffer {
           content: vec![0; buffer_size],
           front_pointer: 0, 
           max_read_size,
        }
    }

    pub fn get_slice_to_read(&mut self) -> &mut [u8] {
        self[front_pointer..(front_pointer + )]
    }
    
    
}