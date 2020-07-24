const DEFAULT_CAPACITY: usize = 8;

pub struct Queue<T: Copy>{
    arr: Vec<T>,
    head: usize,
    size: usize,
    capacity: usize,
}

impl<T: Copy> Queue<T>{
    pub fn new() -> Self{
        Self{
            arr: Vec::with_capacity(DEFAULT_CAPACITY),
            head: 0,
            size: 0,
            capacity: DEFAULT_CAPACITY
        }
    }

    pub fn with_capacity(capacity: usize) -> Self{
        Self{
            arr: Vec::with_capacity(capacity),
            head: 0,
            size: 0,
            capacity
        }
    }

    pub fn push(&mut self, ele: T) -> usize {
        if self.capacity == self.size{
            // Rotate array so that we can insert next element at end
            if self.head != 0 {
                let first = &mut self.arr[..self.head];
                first.reverse();
                let second = &mut self.arr[self.head..];
                second.reverse();
                self.arr.reverse();
            }

            self.capacity *= 2;
            self.head = self.size;
        }

        if self.head >= self.arr.len() {
            self.arr.push(ele);
        } else {
            self.arr[self.head] = ele;
        }

        self.size += 1;
        let head = self.head;
        self.head = (self.head + 1) % self.capacity;
        return head;
    }

    pub fn pop(&mut self) -> Option<T>{
        if self.size == 0 { None }
        else{
            let tail = ((self.head + self.capacity) - self.size) % self.capacity;
            self.size -= 1;
            Some(self.arr[tail])
        }
    }

    pub fn len(&self) -> usize{
        self.size
    }
}

impl<T: Copy + std::fmt::Debug> Queue<T> {
    pub fn print(&self) {
        print!("Size: {} | Capacity: {} :: ", self.size, self.capacity);
        let tail = ((self.head + self.capacity) - self.size) % self.capacity;
        if tail < self.head {
            for i in 0..tail {
                print!(".");
            }
            for i in tail..self.head {
                print!("{:?} ", self.arr[i]);
            }
            for i in self.head..self.capacity{
                print!(".");
            }
        }
        else{
            for i in 0..self.head {
                print!("{:?} ", self.arr[i]);
            }
            for i in self.head..tail {
                print!(".");
            }
            for i in tail..self.capacity{
                print!("{:?} ", self.arr[i]);
            }
        }
        print!("\n");
    }
}