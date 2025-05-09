use crate::Point;

#[derive(Debug, Clone, Copy)]
pub struct Matrix4x4 {
    pub data: [[f32; 4]; 4],
}

impl Matrix4x4 {
    pub fn identity() -> Self {
        Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
    pub fn rotation_around_axis(axis: Point, angle_radians: f32) -> Matrix4x4 {
        let normalized_axis = axis.normalize();
        let x = normalized_axis.x;
        let y = normalized_axis.y;
        let z = normalized_axis.z;

        let cos_theta = angle_radians.cos();
        let sin_theta = angle_radians.sin();
        let one_minus_cos = 1.0 - cos_theta;

        Matrix4x4 {
            data: [
                [
                    cos_theta + x * x * one_minus_cos,
                    x * y * one_minus_cos - z * sin_theta,
                    x * z * one_minus_cos + y * sin_theta,
                    0.0,
                ],
                [
                    y * x * one_minus_cos + z * sin_theta,
                    cos_theta + y * y * one_minus_cos,
                    y * z * one_minus_cos - x * sin_theta,
                    0.0,
                ],
                [
                    z * x * one_minus_cos - y * sin_theta,
                    z * y * one_minus_cos + x * sin_theta,
                    cos_theta + z * z * one_minus_cos,
                    0.0,
                ],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn multiply_point(&self, point: &Point) -> Point {
        let x = self.data[0][0] * point.x
            + self.data[1][0] * point.y
            + self.data[2][0] * point.z
            + self.data[3][0];
        let y = self.data[0][1] * point.x
            + self.data[1][1] * point.y
            + self.data[2][1] * point.z
            + self.data[3][1];
        let z = self.data[0][2] * point.x
            + self.data[1][2] * point.y
            + self.data[2][2] * point.z
            + self.data[3][2];
        let w = self.data[0][3] * point.x
            + self.data[1][3] * point.y
            + self.data[2][3] * point.z
            + self.data[3][3];

        // Perspektivische Division, wenn w != 1.0
        if w != 0.0 {
            Point {
                x: x / w,
                y: y / w,
                z: z / w,
            }
        } else {
            Point { x, y, z }
        }
    }

    pub fn multiply(&self, other: &Matrix4x4) -> Matrix4x4 {
        let mut result = Matrix4x4::identity();
        for i in 0..4 {
            for j in 0..4 {
                result.data[i][j] = (0..4).map(|k| self.data[i][k] * other.data[k][j]).sum();
            }
        }
        result
    }
    pub fn translate(tx: f32, ty: f32, tz: f32) -> Self {
        let mut matrix = Matrix4x4::identity();
        matrix.data[0][3] = tx;
        matrix.data[1][3] = ty;
        matrix.data[2][3] = tz;
        matrix
    }

    pub fn scale(sx: f32, sy: f32, sz: f32) -> Self {
        let mut matrix = Matrix4x4::identity();
        matrix.data[0][0] = sx;
        matrix.data[1][1] = sy;
        matrix.data[2][2] = sz;
        matrix
    }

    pub fn rotate_z(angle: f32) -> Self {
        let mut matrix = Matrix4x4::identity();
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();
        matrix.data[0][0] = cos_theta;
        matrix.data[0][1] = -sin_theta;
        matrix.data[1][0] = sin_theta;
        matrix.data[1][1] = cos_theta;
        matrix
    }

    pub fn rotate_x(angle: f32) -> Self {
        let mut matrix = Matrix4x4::identity();
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();
        matrix.data[1][1] = cos_theta;
        matrix.data[1][2] = -sin_theta;
        matrix.data[2][1] = sin_theta;
        matrix.data[2][2] = cos_theta;
        matrix
    }

    pub fn rotate_y(angle: f32) -> Self {
        let mut matrix = Matrix4x4::identity();
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();
        matrix.data[0][0] = cos_theta;
        matrix.data[0][2] = sin_theta;
        matrix.data[2][0] = -sin_theta;
        matrix.data[2][2] = cos_theta;
        matrix
    }
}
