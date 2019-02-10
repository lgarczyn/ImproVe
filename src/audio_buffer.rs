use std::collections::VecDeque;
use std::sync::mpsc::Receiver;

#[derive(Default)]
pub struct BufferOptions {
    pub resolution: usize,
    pub discard: bool,
    pub overlap: bool,
}

pub struct AudioBuffer {
    options: BufferOptions,
    buffer: VecDeque<f32>,
    receiver: Receiver<Vec<f32>>,
}

impl AudioBuffer {
    pub fn new(receiver: Receiver<Vec<f32>>, options: BufferOptions) -> AudioBuffer {
        AudioBuffer {
            buffer: VecDeque::with_capacity(options.resolution),
            receiver,
            options,
        }
    }

    // Return n elements, n being options.resolution
    // If options.discard is true, overwrite old elements
    // If options.overlap is true, don't delete read elements
    // When receiver dies and data is exhausted, start returning None
    pub fn take(&mut self) -> Option<Vec<f32>> {
        // Set n as the previously received packet resolution
        let n = self.options.resolution;
        // Read all waiting packets
        for packet in self.receiver.try_iter() {
            self.buffer.extend(packet);
        }
        // Make sure buffer contains at least n elements
        while self.buffer.len() < n {
            let recv = self.receiver.recv().ok()?;
            self.buffer.extend(recv);
        }
        // If discard is on, discard surplus data
        if self.options.discard && self.buffer.len() > n {
            self.buffer.drain(0..self.buffer.len() - n);
        }
        // If overlap is allowed, return n oldest elements, and only delete those over the limit
        if self.options.overlap {
            let ret = self.buffer.iter().cloned().take(n).collect();
            // Calculate unneeded data for next batch
            let surplus = self.buffer.len() - n;
            // Cap surplus at n to avoid ignoring data
            let surplus = surplus.min(n);
            // Delete surplus data
            self.buffer.drain(0..surplus);
            Some(ret)
        // If overlap is not allowed, remove them before returning
        } else {
            Some(self.buffer.drain(0..n).collect())
        }
    }
}
