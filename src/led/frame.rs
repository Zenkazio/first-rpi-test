use crate::led::led::LED;

#[derive(Debug, Clone)]
pub struct Frame(pub Vec<LED>);

impl Frame {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn get(&self, index: usize) -> Option<&LED> {
        self.0.get(index)
    }
    pub fn add(&self, other: &Self) -> Self {
        let max = self.len().max(other.len());
        let mut v = Vec::new();
        for i in 0..max {
            let l1 = self.get(i).unwrap_or(&LED(0, 0, 0));
            let l2 = other.get(i).unwrap_or(&LED(0, 0, 0));
            v.push(l1.add(l2));
        }
        Frame(v)
    }
    pub fn shl(&self, mid: usize) -> Self {
        let mut t = self.0.clone();
        t.rotate_left(mid);
        Frame(t)
    }
    pub fn shr(&self, mid: usize) -> Self {
        let mut t = self.0.clone();
        t.rotate_right(mid);
        Frame(t)
    }
    pub fn reverse(&self) -> Self {
        let mut t = self.0.clone();
        t.reverse();
        Frame(t)
    }
    pub fn scale(&self, fac: f32) -> Self {
        let mut v = Vec::new();
        for led in &self.0 {
            v.push(led.scale(fac));
        }
        Frame(v)
    }
}
