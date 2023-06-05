pub struct MinCircularBuffer<Data, const N:usize> 
where
    Data: Copy + Clone
{
    data: [Data; N],
    start_index: usize,
    end_index: usize,
}

impl<Data, const N:usize> MinCircularBuffer<Data, N> 
where Data: Copy + Clone
{
    pub fn new(default_value: Data) -> Self {
        Self {data: [default_value; N], start_index: 0, end_index: 0}
    }

    /// push a data into buffer
    /// 
    /// return false if overflow
    pub fn push(&mut self, data: Data) -> bool{
        self.data[self.end_index] = data;
        self.end_index += 1;
        self.end_index %= N;
        self.end_index != self.start_index
    }

    /// pop a data from buffer
    /// 
    /// return None if empty
    /// return Some(data) elsewise
    pub fn pop(&mut self) -> Option<Data> {
        if self.start_index == self.end_index {
            None
        } else {
            let res = Some(self.data[self.start_index]);
            self.start_index += 1;
            self.start_index %= N;
            res
        }
    }
}