use crate::{point::cross_product, point::dot_product, point::normalize, Matrix4x4, Point};

#[derive(Debug)]
pub struct Camera {
    pub position: Point,         // Position der Kamera
    pub forward: Point,          // Richtung, in die die Kamera schaut
    pub up: Point,               // "Up"-Vektor der Kamera
    pub fov: f32,                // Field of View (FOV), in Grad
    pub aspect_ratio: f32,       // Breite / Höhe des Fensters
    pub near: f32,               // Near-Clipping-Plane
    pub far: f32,                // Far-Clipping-Plane
}

impl Camera {
    pub fn new(position: Point, forward: Point, up: Point, fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            position,
            forward,
            up,
            fov,
            aspect_ratio,
            near,
            far,
        }
    }

    // Funktion zur Erstellung einer View-Matrix (notwendige Transformation)
    pub fn view_matrix(&self) -> Matrix4x4 {
        let forward = normalize(self.forward);
        let right = normalize(cross_product(forward, self.up));
        let up = cross_product(right, forward);

        let tx = -dot_product(right, self.position);
        let ty = -dot_product(up, self.position);
        let tz = dot_product(forward, self.position);

        Matrix4x4 {
            data: [
                [right.x, up.x, -forward.x, 0.0],
                [right.y, up.y, -forward.y, 0.0],
                [right.z, up.z, -forward.z, 0.0],
                [tx, ty, tz, 1.0],
            ],
        }
    }

    // Funktion zur Erstellung einer Projektion-Matrix (zur 2D-Projektion)
    pub fn projection_matrix(&self) -> Matrix4x4 {
        let fov_rad = (self.fov.to_radians() / 2.0).tan();
        Matrix4x4 {
            data: [
                [1.0 / (self.aspect_ratio * fov_rad), 0.0, 0.0, 0.0],
                [0.0, 1.0 / fov_rad, 0.0, 0.0],
                [0.0, 0.0, self.far / (self.far - self.near), 1.0],
                [0.0, 0.0, (-self.far * self.near) / (self.far - self.near), 0.0],
            ],
        }
    }
    pub fn move_forward(&mut self, distance: f32) {
        self.position = self.position - self.forward.normalize() * distance;
    }

    pub fn move_backward(&mut self, distance: f32) {
        self.position = self.position + self.forward.normalize() * distance;
    }

    pub fn strafe_right(&mut self, distance: f32) {
        let right = self.forward.cross(self.up).normalize();
        self.position = self.position + right * distance;
    }

    pub fn strafe_left(&mut self, distance: f32) {
        let right = self.forward.cross(self.up).normalize();
        self.position = self.position - right * distance;
    }

    pub fn move_up(&mut self, distance: f32) {
        self.position = self.position - self.up.normalize() * distance;
    }

    pub fn move_down(&mut self, distance: f32) {
        self.position = self.position + self.up.normalize() * distance;
    }

    pub fn look_right(&mut self, angle_radians: f32) {
        // Rotiert die Kamera um die "Up"-Achse nach rechts.
        let rotation_matrix = Matrix4x4::rotation_around_axis(self.up, -angle_radians);
        self.forward = rotation_matrix.multiply_point(&self.forward).normalize();
    }

    pub fn look_left(&mut self, angle_radians: f32) {
        // Rotiert die Kamera um die "Up"-Achse nach links.
        self.look_right(-angle_radians); // Nach links ist die negative Richtung zu "look_right".
    }

}