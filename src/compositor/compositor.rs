use vello::kurbo::{Affine, Point, Rect};
use vello::peniko::Mix;
use vello::Scene;
use crate::compositor::layer::Layer;

#[derive(Default)]
pub struct Compositor {
    layers: Vec<Layer>,
}

impl Compositor {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    // Add a layer (sorted by z-index)
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
        self.layers.sort_by_key(|l| l.z_index);
    }

    // Update layer properties
    pub fn update_layer(&mut self, id: u32, transform: Option<Affine>, opacity: Option<f32>) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == id) {
            if let Some(t) = transform {
                layer.transform = t;
            }
            if let Some(o) = opacity {
                layer.opacity = o;
            }
        }
    }

    // Compose all layers into a final scene
    pub fn compose(&self, scene: &mut Scene) {
        for layer in &self.layers {
            // let full_rect = Rect::from((Point::new(0.0, 0.0), Point::new(f64::MAX, f64::MAX)));
            let full_rect = Rect::from((Point::new(0.0, 0.0), Point::new(1000.0, 1000.0)));
            scene.push_layer(Mix::Normal, layer.opacity, Affine::IDENTITY, &full_rect);
            scene.append(&layer.content, Some(layer.transform));
            scene.pop_layer();
        }
    }
}
