
use std::sync::Mutex;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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

impl<T: Clone> Clone for CircularBuffer<T> {
    fn clone(&self) -> Self {
        CircularBuffer {
            buffer: self.buffer.clone(),
            head: self.head,
            tail: self.tail,
            size: self.size,
            capacity: self.capacity,
        }
    }
}

type SharedCircularBuffer<T> = Arc<Mutex<CircularBuffer<T>>>;

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

pub fn main_ex2() -> Result<String, Box<dyn std::error::Error>> {
    println!("------------------------------------------------");

    let mut handles = Vec::new();
    let circ_buffer: CircularBuffer<i32> = CircularBuffer::new(100);

    // Writer
    let mut buffer_clone: CircularBuffer<i32> = circ_buffer.clone();
    let join_handle = thread::spawn(move || {
        loop {
            let res = buffer_clone.write(42);
            match res {
                Ok(()) => { println!("wrote to the buffer"); }
                Err(_) => { println!("error writing to buffer"); }
            }
            thread::sleep(Duration::from_secs(2)); 
        }
    });
    handles.push(join_handle);

    // Reader
    let mut buffer_clone: CircularBuffer<i32> = circ_buffer.clone();
    let join_handle = thread::spawn(move || {
        loop {
            let res = buffer_clone.read();
            match res {
                Some(value) => {println!("value: {}", value)}
                _ => {println!("empty buffer")}
            }
            thread::sleep(Duration::from_secs(1)); 
        }
    });
    handles.push(join_handle);

    for handle in handles {
        let res = handle.join();
        match res {
            Ok(()) => {}
            Err(_) => {println!("error joining thread")}
        }
    }

    Ok("END".to_string())
}
