use cgmath::{perspective, InnerSpace, Matrix4, Point3, Rad, Vector3};
use instant::Duration;
use winit::{
    dpi::PhysicalPosition,
    event::MouseScrollDelta,
};

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[derive(Debug)]
pub struct Camera {
    pub focus: Point3<f32>,
    yaw: f32,
    dist: f32,
    height: f32,
}

impl Camera {
    pub fn new<P: Into<Point3<f32>>>(focus: P, yaw: f32, dist: f32, height: f32) -> Self {
        Self {
            focus: focus.into(),
            yaw,
            dist,
            height,
        }
    }

    pub fn position(&self) -> Point3<f32> {
        Point3::new(
            self.focus.x + (self.dist * self.yaw.cos()),
            self.focus.y + (self.height),
            self.focus.z + (self.dist * self.yaw.sin()),
        )
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.position(), self.focus, Vector3::unit_y())
    }
}

// We need this for rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    // We can't use cgmath with bytemuck directly so we'll have to convert the
    // Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.focus.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

const VERTICAL_BOOST: f32 = 5.0;
const MIN_CAMERA_DIST: f32 = 5.0;

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    zoom_change: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            zoom_change: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn forward(&mut self, amount: f32) {
        self.amount_forward = amount;
    }

    pub fn left(&mut self, amount: f32) {
        self.amount_left = amount;
    }

    pub fn right(&mut self, amount: f32) {
        self.amount_right = amount;
    }

    pub fn back(&mut self, amount: f32) {
        self.amount_backward = amount;
    }

    pub fn up(&mut self, amount: f32) {
        self.amount_up = amount;
    }

    pub fn down(&mut self, amount: f32) {
        self.amount_down = amount;
    }

    pub fn turn(&mut self, dx: f64, dy: f64) {
        self.rotate_horizontal = dx as f32;
        self.rotate_vertical = dy as f32;
    }

    pub fn zoom(&mut self, delta: &MouseScrollDelta) {
        self.zoom_change = -match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, z) => z * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = -Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = -Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.focus += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.focus += right * (self.amount_right - self.amount_left) * self.speed * dt;

        camera.height += VERTICAL_BOOST * self.rotate_vertical * dt;
        camera.yaw += self.rotate_horizontal * dt;

        // Move in/out (aka. "zoom")
        // let pitch = (camera.dist / camera.height).atan();

        // camera.height += pitch.sin() * self.scroll * self.speed * self.sensitivity * dt;
        let h_d_ratio = camera.height / camera.dist;
        let dist_change = self.zoom_change * self.speed * self.sensitivity * VERTICAL_BOOST * dt;
        camera.dist += dist_change;
        camera.height += dist_change * h_d_ratio;

        // camera.height = camera.height.max(0.0);
        camera.dist = camera.dist.max(MIN_CAMERA_DIST);

        // Move up/down
        camera.focus.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        self.zoom_change = 0.0;
    }
}
