use winit::dpi::PhysicalSize;

use super::vertex::UIVertex;

pub trait Component {
    fn positioner(&self, canvas_size: PhysicalSize<u32>) -> Vec<UIVertex>;
}