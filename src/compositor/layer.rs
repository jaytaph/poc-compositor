use vello::kurbo::Affine;
use vello::Scene;

pub struct Layer {
    pub id: u32,
    pub content: Scene,
    pub transform: Affine,
    pub opacity: f32,
    pub z_index: i32,
}

impl Layer {
    pub fn new(id: u32, content: Scene, transform: Affine, opacity: f32, z_index: i32) -> Self {
        Self {
            id,
            content,
            transform,
            opacity,
            z_index,
        }
    }
}