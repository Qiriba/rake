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

    // New fields for movement
    pub velocity: Point,         // Current velocity of the camera
    pub acceleration: f32,       // Acceleration for forward/backward/strafe
    pub gravity: f32,            // Gravity applied during jumps
    pub vertical_velocity: f32,  // Velocity for jumping
    pub is_jumping: bool,        // Whether the camera is currently jumping

    // Look
    pub look_sensitivity: f32,      // Sensitivity for looking around
    pub yaw: f32,                   // Horizontal rotation
    pub pitch: f32,                 // Vertical rotation

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
            acceleration: 1.0,      // Increased movement speed
            gravity: -9.8,
            vertical_velocity: 0.0,
            is_jumping: false,

            // Look
            look_sensitivity: 0.005, // Fine-tune this
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
        // Calculate new forward direction using yaw and pitch
        self.forward = Point::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
            .normalize();
    }

    pub fn look_around(&mut self, delta_x: f32, delta_y: f32) {
        // Adjust yaw and pitch based on mouse movement or control input
        self.yaw += delta_x * self.look_sensitivity;
        self.pitch += delta_y * self.look_sensitivity;

        // Clamp the pitch to prevent looking too far up or down
        self.pitch = self.pitch.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

        // Recalculate forward direction
        self.update_forward();
    }

    pub fn update_movement(&mut self, delta_time: f32, keys: &[bool; 256], mouse_delta: (f32, f32)) {
        // Constants for movement and physics
        let sv_accelerate = 10.0;       // Ground acceleration
        let sv_air_accelerate = 2.0;   // Air acceleration
        let sv_gravity = 800.0;        // Gravity force (units/s²)
        let sv_jump_speed = 300.0;     // Initial upward velocity when jumping
        let sv_maxspeed = 320.0;       // Maximum player movement speed on the ground
        let sv_max_air_speed = 160.0;  // Maximum player speed in the air
        let sv_ground_friction = 6.0; // Friction applied on the ground

        // Normalize movement vectors
        let forward = self.forward.normalize();
        let right = self.forward.cross(self.up).normalize();
        let up = self.up.normalize();

        let mut move_dir = Point::new(0.0, 0.0, 0.0);

        // Process WASD for movement
        if keys[b'W' as usize] {
            move_dir = move_dir - forward;
        }
        if keys[b'S' as usize] {
            move_dir = move_dir + forward;
        }
        if keys[b'D' as usize] {
            move_dir = move_dir + right;
        }
        if keys[b'A' as usize] {
            move_dir = move_dir - right;
        }

        // Determine if we're on the ground
        let is_on_ground = self.position.y <= 0.1;

        if is_on_ground {
            // RESET JUMP STATE
            self.is_jumping = false;
            self.vertical_velocity = 0.0; // Stop vertical movement upon landing

            // Apply friction
            if self.velocity.magnitude() > 0.0 {
                let speed = self.velocity.magnitude();
                let control = speed.min(sv_maxspeed);
                let friction = sv_ground_friction * delta_time * control;

                // Decelerate by friction, but don't reverse direction
                self.velocity = self.velocity * (1.0 - friction / speed).max(0.0);
            }

            // Apply ground movement
            if move_dir.magnitude() > 0.0 {
                move_dir = move_dir.normalize();
                let wish_vel = move_dir * sv_maxspeed;
                let accel = sv_accelerate * delta_time;
                self.velocity = self.velocity + (wish_vel - self.velocity).clamp_length(accel);
            }

            // Handle jumping (if the player hits the jump key)
            if keys[b'V' as usize] {
                self.is_jumping = true;            // Set jumping state
                self.vertical_velocity = sv_jump_speed; // Apply upward jump velocity
            }
        } else {
            // In the air (falling or jumping)
            if move_dir.magnitude() > 0.0 {
                move_dir = move_dir.normalize();
                let wish_vel = move_dir * sv_max_air_speed;
                let accel = sv_air_accelerate * delta_time;
                self.velocity = self.velocity + (wish_vel - self.velocity).clamp_length(accel);
            }

            // Apply gravity
            self.vertical_velocity -= sv_gravity * delta_time;
        }

        // Apply vertical velocity to control jumping and falling
        self.position.y += self.vertical_velocity * delta_time;

        // Clamp velocity for horizontal movement
        if is_on_ground {
            self.velocity = self.velocity.clamp_length(sv_maxspeed);
        } else {
            self.velocity = self.velocity.clamp_length(sv_max_air_speed);
        }

        // Apply horizontal velocity to position
        self.position = self.position + self.velocity * delta_time;

        // Prevent falling below the ground
        if self.position.y <= 0.0 {
            self.position.y = 0.0;
            self.vertical_velocity = 0.0;
        }

        // Handle camera look based on mouse input
        //self.look_around(mouse_delta.0, mouse_delta.1);
    }




}