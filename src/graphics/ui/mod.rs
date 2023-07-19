use std::collections::HashMap;

use wgpu::BindGroup;

use self::component::Component;

pub mod model;
pub mod component;
pub mod buttons;

pub struct UI_manager {
    components: Vec<Box<dyn Component>>,
    bind_groups: HashMap<String, BindGroup>,
}