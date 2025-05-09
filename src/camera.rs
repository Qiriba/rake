use crate::{Matrix4x4, Point, point::cross_product, point::dot_product, point::normalize};

#[derive(Debug)]
pub struct Camera {
    pub position: Point,   // Position der Kamera
    pub forward: Point,    // Richtung, in die die Kamera schaut
    pub up: Point,         // "Up"-Vektor der Kamera
    pub fov: f32,          // Field of View (FOV), in Grad
    pub aspect_ratio: f32, // Breite / Höhe des Fensters
    pub near: f32,         // Near-Clipping-Plane
    pub far: f32,          // Far-Clipping-Plane

    // New fields for movement
    pub velocity: Point,        // Current velocity of the camera
    pub acceleration: f32,      // Acceleration for forward/backward/strafe
    pub gravity: f32,           // Gravity applied during jumps
    pub vertical_velocity: f32, // Velocity for jumping
    pub is_jumping: bool,       // Whether the camera is currently jumping

    // Look
    pub look_sensitivity: f32, // Sensitivity for looking around
    pub yaw: f32,              // Horizontal rotation
    pub pitch: f32,            // Vertical rotation
}

impl Camera {
    pub fn new(
        position: Point,
        forward: Point,
        up: Point,
        fov: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            position,
            forward,
            up,
            fov,
            aspect_ratio,
            near,
            far,

            // Movement
            velocity: Point::new(0.0, 0.0, 0.0),
            acceleration: 1.0, // Increased movement speed
            gravity: 80.0,
            vertical_velocity: 0.0,
            is_jumping: false,

            // Look
            look_sensitivity: 0.001, // Fine-tune this
            yaw: 0.0,
            pitch: 0.0,
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
                [
                    0.0,
                    0.0,
                    (-self.far * self.near) / (self.far - self.near),
                    0.0,
                ],
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

    pub fn look_left(&mut self) {
        // Rotate the forward vector counter-clockwise around the up vector (yaw left)
        let rotation_matrix = Matrix4x4::rotation_around_axis(self.up, self.look_sensitivity);
        self.forward = rotation_matrix.multiply_point(&self.forward).normalize();
    }

    pub fn look_right(&mut self) {
        // Rotate the forward vector clockwise around the up vector (yaw right)
        let rotation_matrix = Matrix4x4::rotation_around_axis(self.up, -self.look_sensitivity);
        self.forward = rotation_matrix.multiply_point(&self.forward).normalize();
    }

    pub fn update_forward(&mut self) {
        // Calculate forward vector
        self.forward.x = self.yaw.cos() * self.pitch.cos();
        self.forward.y = self.pitch.sin();
        self.forward.z = self.yaw.sin() * self.pitch.cos();

        // Stabilize extremely small values
        if self.forward.x.abs() < 1e-6 {
            self.forward.x = 0.0; // Snap small x to 0
        }
        if self.forward.z.abs() < 1e-6 {
            self.forward.z = 0.0; // Snap small z to 0
        }

        // Normalize the forward vector to prevent precision drift
        self.forward = self.forward.normalize();

        // Recalculate right and up vectors for stability
        let right = self.forward.cross(Point::new(0.0, 1.0, 0.0)).normalize();
        self.up = right.cross(self.forward).normalize();
    }

    pub fn look_around(&mut self, delta_x: f32, delta_y: f32) {
        // Skip updates if there's no input
        if delta_x.abs() < 1e-6 && delta_y.abs() < 1e-6 {
            return;
        }

        // Adjust yaw and pitch based on mouse movement or control input
        self.yaw -= delta_x * self.look_sensitivity;
        self.pitch += delta_y * self.look_sensitivity;

        // Clamp the pitch to prevent looking too far up or down
        self.pitch = self
            .pitch
            .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

        // Recalculate forward direction
        self.update_forward();
    }

    pub fn update_movement(
        &mut self,
        delta_time: f32,
        keys: &[bool; 256],
        mouse_delta: (f32, f32),
    ) {
        // Constants for movement and physics
        // Constants/configurations
        let sv_maxspeed = 320.0; // Max speed on ground
        let sv_air_maxspeed = 950.0; // Max speed while air strafing
        let sv_accelerate = 5.5; // Ground acceleration
        let sv_air_accelerate = 12.0; // Air strafing acceleration
        let sv_friction = 0.6; // Ground friction
        let jump_strength = 27.0; // Jump power
        let air_drag = 0.01; // Air drag to slow player slightly in air
        let diagonal_speed_scale = 1.1; // Slightly faster diagonal movement
        let strafe_boost_factor = 0.02; // Small boost for strafing on the ground

        // Get movement direction from keys (W/A/S/D)
        let mut move_dir = Point::new(0.0, 0.0, 0.0);
        if keys['W' as usize] {
            move_dir = move_dir - self.forward;
        }
        if keys['S' as usize] {
            move_dir = move_dir + self.forward;
        }
        if keys['A' as usize] {
            move_dir = move_dir - self.forward.cross(self.up); // Left strafe
        }
        if keys['D' as usize] {
            move_dir = move_dir + self.forward.cross(self.up); // Right strafe
        }

        // Normalize the movement direction, ignore vertical (Y) component
        move_dir.y = 0.0; // Ensure no vertical influence on movement direction
        move_dir = move_dir.normalize(); // Avoid NaN if direction is zero

        // Ground state
        if !self.is_jumping && self.is_on_ground() {
            // Pre-strafe mechanics and ground acceleration
            if move_dir.magnitude() > 0.0 {
                // Apply ground movement
                let wish_vel = move_dir * sv_maxspeed * diagonal_speed_scale;
                let accel = sv_accelerate * delta_time;
                self.velocity = self.velocity + (wish_vel - self.velocity) * accel * delta_time;

                // Add a slight boost for strafing
                if keys['A' as usize] || keys['D' as usize] {
                    self.velocity = self.velocity * (1.0 + (strafe_boost_factor * delta_time));
                }
            }

            // Apply friction to slow down slightly when not jumping
            if !keys['V' as usize] {
                self.velocity = self.velocity * sv_friction;
            } else {
                self.is_jumping = true; // Prepare for jump
            }

            // Jump mechanics
            if keys['V' as usize] {
                self.vertical_velocity = jump_strength; // Apply vertical velocity
            }
        } else {
            // Air strafing logic
            if move_dir.magnitude() > 0.0 {
                let air_accel = sv_air_accelerate * delta_time;
                let wish_vel = move_dir * sv_air_maxspeed;
                self.velocity = self.velocity + (wish_vel - self.velocity).clamp_length(air_accel);
            }

            // Apply slight air drag for balance
            self.velocity = self.velocity * (1.0 - (air_drag * delta_time));

            // Clamp speed to max air speed
            if self.velocity.magnitude() > sv_air_maxspeed {
                self.velocity = self.velocity.normalize() * sv_air_maxspeed;
            }

            // Apply gravity to vertical velocity
            self.vertical_velocity -= self.gravity * delta_time;

            // Update position based on velocity
            self.position.y += self.vertical_velocity * delta_time;

            // Check for landing
            if self.position.y < 0.0 {
                self.position.y = 0.0;
                self.vertical_velocity = 0.0;
                self.is_jumping = false;
            }
        }

        // Update position based on horizontal velocity
        self.position.x += self.velocity.x * delta_time;
        self.position.z += self.velocity.z * delta_time;

        self.look_around(mouse_delta.0, mouse_delta.1);
    }

    fn is_on_ground(&self) -> bool {
        self.position.y <= 0.0
    }
}
