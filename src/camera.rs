use cgmath::{Deg, Matrix4, Point3, Vector3};
use cgmath::SquareMatrix;

#[rustfmt::skip]
   pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::from_cols(
       cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
       cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
       cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
       cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
   );

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(Deg(self.fov_y), self.aspect, self.z_near, self.z_far);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_projection: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new(camera: &Camera) -> Self {
        Self { view_projection: Matrix4::identity().into() }
    }

    pub fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = camera.build_view_projection_matrix().into();
    }
}

