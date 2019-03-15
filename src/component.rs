#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Component {
    pub intensity: f32,
    pub frequency: f32,
}

impl Component {
    #[allow(dead_code)]
    pub fn amplitude(self) -> f32 {
        self.intensity.sqrt()
    }
}

impl Eq for Component {}

impl Ord for Component {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
