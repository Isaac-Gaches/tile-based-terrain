pub struct Transform{
    pub translation: [f32;2],
    pub rotation: f32,
}

impl Transform{
    pub fn new(translation: [f32;2])-> Self{
        Self{
            translation,
            rotation: 0.0,
        }
    }
}