

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Frequency {
	pub intensity:f32,
	pub value:f32,
}

impl Frequency {
	#[allow(dead_code)]
	pub fn amplitude(self) -> f32 {
		self.intensity.sqrt()
	}
}

impl Eq for Frequency { }

impl Ord for Frequency {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.partial_cmp(other).unwrap()
	}
}
