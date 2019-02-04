

#[derive(Clone, Copy, Debug, Default)]
pub struct Frequency {
	pub value:f32,
	pub intensity:f32
}

impl Frequency {
	#[allow(dead_code)]
	pub fn amplitude(&self) -> f32 {
		self.intensity.sqrt()
	}
}
