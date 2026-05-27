use glam::{Mat4, Quat, Vec3};

use crate::event::MotionEvent;

/// Accumulator for position + quaternion orientation from motion events.
/// Equivalent to C `spnav_posrot`.
#[derive(Debug, Clone, Copy)]
pub struct PositionRot {
    pub pos: Vec3,
    pub rot: Quat,
}

impl Default for PositionRot {
    fn default() -> Self {
        Self::new()
    }
}

impl PositionRot {
    /// Initialize to zero position, identity rotation
    pub fn new() -> Self {
        Self {
            pos: Vec3::ZERO,
            rot: Quat::IDENTITY,
        }
    }

    /// Accumulate a motion event as object-space movement.
    /// Equivalent to C `spnav_posrot_moveobj`.
    pub fn move_obj(&mut self, ev: &MotionEvent) {
        self.pos.x += ev.x as f32 * 0.001;
        self.pos.y += ev.y as f32 * 0.001;
        self.pos.z -= ev.z as f32 * 0.001;

        let len = (ev.rx as f32).hypot((ev.ry as f32).hypot(ev.rz as f32));
        if len != 0.0 {
            let x = ev.rx as f32 / len;
            let y = ev.ry as f32 / len;
            let z = -(ev.rz as f32) / len;
            let delta = Quat::from_axis_angle(Vec3::new(x, y, z), len * 0.001);
            self.rot = delta * self.rot;
        }
    }

    /// Accumulate a motion event as view-space movement.
    /// Equivalent to C `spnav_posrot_moveview`.
    pub fn move_view(&mut self, ev: &MotionEvent) {
        let len = (ev.rx as f32).hypot((ev.ry as f32).hypot(ev.rz as f32));
        if len != 0.0 {
            let x = -(ev.rx as f32) / len;
            let y = -(ev.ry as f32) / len;
            let z = ev.rz as f32 / len;
            let delta = Quat::from_axis_angle(Vec3::new(x, y, z), len * 0.001);
            self.rot = delta * self.rot;
        }

        let trans = Vec3::new(
            -(ev.x as f32) * 0.001,
            -(ev.y as f32) * 0.001,
            ev.z as f32 * 0.001,
        );
        self.pos += self.rot * trans;
    }

    /// Build a model/world matrix (OpenGL column-major).
    /// Equivalent to C `spnav_matrix_obj`.
    pub fn to_model_matrix(&self) -> Mat4 {
        Mat4::from_quat(self.rot) * Mat4::from_translation(self.pos)
    }

    /// Build a view matrix.
    /// Equivalent to C `spnav_matrix_view`.
    pub fn to_view_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.pos) * Mat4::from_quat(self.rot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let pr = PositionRot::new();
        assert_eq!(pr.pos, Vec3::ZERO);
        assert!(pr.rot.abs_diff_eq(Quat::IDENTITY, 1e-6));
    }

    #[test]
    fn test_move_obj_translation() {
        let mut pr = PositionRot::new();
        let ev = MotionEvent {
            x: 1000,
            y: 0,
            z: 0,
            rx: 0,
            ry: 0,
            rz: 0,
            period: 0,
        };
        pr.move_obj(&ev);
        assert!((pr.pos.x - 1.0).abs() < 1e-6);
        assert_eq!(pr.pos.y, 0.0);
        assert_eq!(pr.pos.z, 0.0);
    }

    #[test]
    fn test_model_matrix_is_translation_only() {
        let mut pr = PositionRot::new();
        pr.pos = Vec3::new(1.0, 2.0, 3.0);
        let mat = pr.to_model_matrix();
        // Translation components in OpenGL column-major
        assert!((mat.w_axis.x - 1.0).abs() < 1e-6);
        assert!((mat.w_axis.y - 2.0).abs() < 1e-6);
        assert!((mat.w_axis.z - 3.0).abs() < 1e-6);
    }
}
