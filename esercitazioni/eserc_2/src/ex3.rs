#[derive(Debug, PartialEq)]
pub enum Err {
    Full,
}

pub struct CircularBuffer<T> { 
    buffer: Vec<Option<T>>,
    head: usize,
    tail: usize,
    size: usize,
    capacity: usize,
}

impl<T> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        CircularBuffer {
            buffer: (0..capacity).map(|_| None).collect(),
            head: 0,
            tail: 0,
            size: 0,
            capacity,
        }
    }

    pub fn write(&mut self, item: T) -> Result<(), Err> {
        if self.size == self.capacity {
            return Err(Err::Full)
        }
        self.buffer[self.tail] = Some(item);
        self.tail = (self.tail + 1) % self.capacity; 
        self.size += 1;
        Ok(())
    }

    pub fn read(&mut self) -> Option<T> {
        if self.size == 0 {
            return None
        }
        let value = self.buffer[self.head].take();
        self.head = (self.head + 1) % self.capacity;
        self.size -= 1;
        value
    }

    pub fn clear(&mut self) {
        for slot in self.buffer.iter_mut() {
            *slot = None;
        }
        self.head = 0;
        self.tail = 0;
        self.size = 0;
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn overwrite(&mut self, item: T) {
        if self.size == self.capacity {
            // buffer pieno
            self.buffer[self.head] = Some(item);
            self.head = (self.head + 1) % self.capacity;
            self.tail = (self.tail + 1) % self.capacity;
        } else {
            self.buffer[self.tail] = Some(item);
            self.tail = (self.tail + 1) % self.capacity;
            self.size += 1;
        }
    }

    pub fn make_contiguous(&mut self) {
        if self.head == 0 || self.size == 0 {
            return;
        }

        let mut new_buffer: Vec<Option<T>> = (0..self.capacity).map(|_| None).collect();
        let mut new_index = 0;

        let mut current = self.head;
        for _ in 0..self.size {
            new_buffer[new_index] = self.buffer[current].take();
            current = (current + 1) % self.capacity;
            new_index += 1;
        }

        self.buffer = new_buffer;
        self.head = 0;
        self.tail = self.size % self.capacity;
    }
}

// Buffer Circolare
pub fn main_ex3() -> Result<String, Box<dyn std::error::Error>> {
    Ok("OK".to_string())
}


// -------------------- TESTS ----------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_check_size() {
        let mut buf = CircularBuffer::new(3);
        assert_eq!(buf.size(), 0);
        buf.write(10).unwrap();
        assert_eq!(buf.size(), 1);
    }

    #[test]
    fn insert_and_read_same_value() {
        let mut buf = CircularBuffer::new(3);
        buf.write(42).unwrap();
        assert_eq!(buf.read(), Some(42));
        assert_eq!(buf.size(), 0);
    }

    #[test]
    fn insert_multiple_and_read_all() {
        let mut buf = CircularBuffer::new(3);
        buf.write(1).unwrap();
        buf.write(2).unwrap();
        buf.write(3).unwrap();
        assert_eq!(buf.read(), Some(1));
        assert_eq!(buf.read(), Some(2));
        assert_eq!(buf.read(), Some(3));
        assert_eq!(buf.read(), None);
    }

    #[test]
    fn head_and_tail_wraparound() {
        let mut buf = CircularBuffer::new(2);
        buf.write(1).unwrap();
        buf.write(2).unwrap();
        assert!(buf.write(3).is_err()); // pieno
        assert_eq!(buf.read(), Some(1));
        buf.write(3).unwrap(); // tail ritorna a zero
        assert_eq!(buf.read(), Some(2));
        assert_eq!(buf.read(), Some(3));
    }

    #[test]
    fn read_from_empty_buffer() {
        let mut buf: CircularBuffer<i32> = CircularBuffer::new(3);
        assert_eq!(buf.read(), None);
    }

    #[test]
    fn write_to_full_buffer_returns_error() {
        let mut buf = CircularBuffer::new(2);
        buf.write(1).unwrap();
        buf.write(2).unwrap();
        assert!(buf.write(3).is_err());
    }

    #[test]
    fn overwrite_on_full_buffer() {
        let mut buf = CircularBuffer::new(2);
        buf.write(1).unwrap();
        buf.write(2).unwrap();
        buf.overwrite(3); // sovrascrive il più vecchio
        assert_eq!(buf.read(), Some(2));
        assert_eq!(buf.read(), Some(3));
    }

    #[test]
    fn make_contiguous_works() {
        let mut buf = CircularBuffer::new(4);
        buf.write(1).unwrap();
        buf.write(2).unwrap();
        buf.write(3).unwrap();
        buf.read(); // head avanza
        buf.write(4).unwrap();
        buf.write(5).unwrap(); // tail wrap-around
        buf.make_contiguous();
        // Ora deve essere contiguo con head = 0
        assert_eq!(buf.read(), Some(2));
        assert_eq!(buf.read(), Some(3));
        assert_eq!(buf.read(), Some(4));
        assert_eq!(buf.read(), Some(5));
    }
}
