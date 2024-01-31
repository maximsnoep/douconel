use std::fmt::Debug;

#[derive(Default, Clone, Debug)]
pub struct Memory<T> {
    buffer: Vec<T>,
    free: Vec<usize>,
}

impl<T: Debug + Clone> Memory<T> {
    pub fn new() -> Self {
        Self {
            buffer: Vec::<T>::new(),
            free: Vec::new(),
        }
    }

    pub fn items(&self) -> Vec<T> {
        self.buffer.clone()
    }

    pub fn debug_print(&self) {
        print!("buffer: ");
        for x in self.buffer.iter() {
            print!("* ");
        }
        println!("");

        print!("free: ");
        for x in self.free.iter() {
            print!("{:?} ", x);
        }
        println!("");
    }

    pub fn alloc(&mut self, value: T) -> usize {
        if let Some(index) = self.free.pop() {
            self.buffer[index] = value;
            index
        } else {
            self.buffer.push(value);
            self.buffer.len() - 1
        }
    }

    pub fn dealloc(&mut self, ptr: usize) {
        self.free.push(ptr);
    }

    pub fn deref(&self, ptr: usize) -> &T {
        &self.buffer[ptr]
    }

    pub fn deref_mut(&mut self, ptr: usize) -> &mut T {
        &mut self.buffer[ptr]
    }
}
