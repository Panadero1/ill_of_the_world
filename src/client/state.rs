use std::time::Instant;

use cgmath::{InnerSpace, Rotation3, Zero};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
};

use crate::{
    graphics::{self, instance::Instance, state::Graphics, ui, PageRes},
    world::World,
};

const NUM_INSTANCES_PER_ROW: u32 = 10;
const SPACE_BETWEEN: f32 = 3.0;

pub struct State {
    world: World,
    start: Instant,
    last_frame: Instant,
    exit: bool,
    last_cur_pos: PhysicalPosition<f64>,
    is_clicking: bool,
}

impl State {
    pub fn new() -> State {
        State {
            world: World::empty(),
            start: Instant::now(),
            last_frame: Instant::now(),
            exit: false,
            last_cur_pos: (0.0, 0.0).into(),
            is_clicking: false,
        }
    }
}

impl graphics::Page for State {
    fn init(&mut self, gr: &mut Graphics) {
        gr.add_model_ui("BR corner", "happy-tree.png", |canvas_size| {
            ui::model::rect_vertices(
                canvas_size,
                PhysicalSize::new(200, 200),
                PhysicalPosition::new(
                    (canvas_size.width as i32) - 200,
                    (canvas_size.height as i32) - 200,
                ),
            )
        });

        gr.add_model_ui("TL corner", "happy-tree.png", |canvas_size| {
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

        // state.add_model_3d_instanced("cube", "cube.obj", instances);

        gr.add_model_3d_instanced("minion", "minion.obj", instances);
        // state.add_model_3d("aa", "cube1.obj");
    }

    fn update(&mut self, gr: &mut Graphics) -> PageRes {
        let now = Instant::now();
        let dt = now - self.last_frame;
        self.last_frame = now;

        let light_pos = [(now - self.start).as_secs_f32().sin() * 10.0, 10.0, 0.0];

        gr.position_light(light_pos);

        gr.update_cam(dt);

        if self.exit {
            PageRes::Exit
        } else {
            PageRes::NoOp
        }
    }

    fn on_exit(&mut self) {}

    fn event(&mut self, gr: &mut Graphics, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => self.match_keys(input, gr),
            WindowEvent::CursorMoved { position, .. } => self.process_mouse(position, gr),
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => self.is_clicking = *state == ElementState::Pressed,
            WindowEvent::MouseWheel { delta, .. } => gr.m3d_mgr.camera_control().zoom(delta),
            WindowEvent::ModifiersChanged(m) => {
                gr.camera_control().down(if m.shift() { 1.0 } else { 0.0 })
            }
            _ => (),
        }
    }
}

impl State {
    fn match_keys(&mut self, input: &KeyboardInput, gr: &mut Graphics) {
        match input {
            KeyboardInput {
                state,
                virtual_keycode: Some(key),
                ..
            } => {
                let amount = if *state == ElementState::Pressed {
                    1.0
                } else {
                    0.0
                };

                match key {
                    // key pressed
                    VirtualKeyCode::Escape if *state == ElementState::Released => self.exit = true,
                    VirtualKeyCode::W => gr.camera_control().forward(amount),
                    VirtualKeyCode::A => gr.camera_control().left(amount),
                    VirtualKeyCode::S => gr.camera_control().back(amount),
                    VirtualKeyCode::D => gr.camera_control().right(amount),
                    VirtualKeyCode::Space => gr.camera_control().up(amount),
                    _ => (),
                }
            }
            _ => (),
        }
    }

    fn process_mouse(&mut self, position: &PhysicalPosition<f64>, gr: &mut Graphics) {
        let dx = position.x - self.last_cur_pos.x;
        let dy = position.y - self.last_cur_pos.y;
        self.last_cur_pos = *position;
        if self.is_clicking {
            gr.m3d_mgr.camera_control().turn(dx, dy);
        }
    }
}
