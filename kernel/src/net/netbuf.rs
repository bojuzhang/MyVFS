#[derive(Debug, Clone)]
pub struct NetBuf {
    pub data: Vec<u8>,
    pub len: usize,
}

impl NetBuf {
    pub fn new(data: Vec<u8>) -> Self {
        let len = data.len();
        Self { data, len }
    }

    pub fn frame(&self) -> &[u8] {
        &self.data[..self.len]
    }
}
