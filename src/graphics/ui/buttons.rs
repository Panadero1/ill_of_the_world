use winit::dpi::PhysicalSize;

use super::{component::Component, vertex::UIVertex};

// todo: move this somewhere else
pub struct StartButton {

}
impl Component for StartButton {
    fn positioner(&self, canvas_size: PhysicalSize<u32>) -> Vec<UIVertex> {
        todo!()
    }
}