use wgpu::SurfaceError;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use self::state::Graphics;

// Note
// The rule of thumb for alignment with WGSL structs is field alignments are always powers of 2
// For example, a vec3 may only have 3 float fields giving it a size of 12
// The alignment will be bumped up to the next power of 2 being 16
// This means that you have to be more careful with how you layout your struct

pub mod camera;
pub mod instance;
mod light;
pub mod m_3d;
pub mod model;
pub mod pipeline;
mod resources;
pub mod state;
pub mod texture;
pub mod ui;

pub enum PageRes {
    NoOp,
    Switch(Box<dyn Page>),
    Exit,
}

pub async fn run(mut page: Box<dyn Page>) {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut gr = Graphics::new(&window).await;

    page.init(&mut gr);

    fn exit_procedure(page: &mut Box<dyn Page>, control_flow: &mut ControlFlow) {
        page.on_exit();
        *control_flow = ControlFlow::Exit;
    }

    fn process_res(page: &mut Box<dyn Page>, res: PageRes, cf: &mut ControlFlow) {
        match res {
            PageRes::NoOp => (),
            PageRes::Exit => exit_procedure(page, cf),
            PageRes::Switch(p) => *page = p,
        }
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => {
                    exit_procedure(&mut page, control_flow);
                }
                WindowEvent::Resized(physical_size) => {
                    gr.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    gr.resize(**new_inner_size);
                }
                _ => page.event(&mut gr, event),
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let res = page.update(&mut gr);

                process_res(&mut page, res, control_flow);

                match gr.render() {
                    Ok(_) => (),
                    // Reconfigure the surface if lost
                    Err(SurfaceError::Lost) => gr.resize(gr.size),
                    // The system is out of memory, we should probably quit
                    Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually request it
                window.request_redraw();
            }
            _ => (),
        }
    });
}

pub trait Page {
    fn init(&mut self, gr: &mut Graphics);
    fn update(&mut self, gr: &mut Graphics) -> PageRes;
    fn on_exit(&mut self);
    fn event(&mut self, gr: &mut Graphics, event: &WindowEvent);
}
