use std::sync::mpsc::Receiver;
use std::iter::Empty;

pub struct AudioBuffer {
	buffer:Vec<f32>,
	receiver:Receiver<Vec<f32>>,
}

impl AudioBuffer {
	pub fn new(receiver:Receiver<Vec<f32>>) -> AudioBuffer {
		AudioBuffer {
			receiver,
			buffer: vec![],
		}
	}

	pub fn take(&mut self, n:usize) -> Vec<f32>
	{
		//Extend buffer until len is at least equal to n
		while self.buffer.len() < n
		{
			self.buffer.extend(self.receiver.recv().unwrap());
		}
		//return the first n elements, and remove them from the buffer
		self.buffer.splice(..n, Empty::default()).collect()
	}
}
