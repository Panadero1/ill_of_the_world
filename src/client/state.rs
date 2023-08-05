use cgmath::{Rotation3, Zero, InnerSpace};
use winit::dpi::{PhysicalPosition, PhysicalSize};

use crate::{
    graphics::{self, ui, instance::Instance},
    world::World,
};

const NUM_INSTANCES_PER_ROW: u32 = 10;
const SPACE_BETWEEN: f32 = 3.0;

pub struct State {
    world: World,
}

impl State {
    pub fn new() -> State {
        State {
            world: World::empty(),
        }
    }
}

impl graphics::Modifier for State {
    fn init_state(&self, state: &mut graphics::state::State) {
        state.add_model_ui("BR corner", "happy-tree.png", |canvas_size| {
            ui::model::rect_vertices(
                canvas_size,
                PhysicalSize::new(200, 200),
                PhysicalPosition::new(
                    (canvas_size.width as i32) - 200,
                    (canvas_size.height as i32) - 200,
                ),
            )
        });

        state.add_model_ui("TL corner", "happy-tree.png", |canvas_size| {
            ui::model::rect_vertices(
                canvas_size,
                PhysicalSize::new(200, 200),
                PhysicalPosition::new(0, 0),
            )
        });

        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = cgmath::Vector3 { x, y: 0.0, z };

                    let rotation = if position.is_zero() {
                        // This is needed so an object at origin won't get scaled to zero
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        state.add_model_3d_instanced("cube", "cube.obj", instances);
    }
}
