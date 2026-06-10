//! Integration tests for PositionRot math utilities.

use glam::{Quat, Vec3};
use spnav_rs::event::MotionEvent;
use spnav_rs::PositionRot;

// ─── Initialization tests ───────────────────────────────────────────────────

#[test]
fn new_starts_at_zero() {
    let pr = PositionRot::new();
    assert_eq!(pr.pos, Vec3::ZERO);
    assert!(pr.rot.abs_diff_eq(Quat::IDENTITY, 1e-6));
}

#[test]
fn default_starts_at_zero() {
    let pr = PositionRot::default();
    assert_eq!(pr.pos, Vec3::ZERO);
    assert!(pr.rot.abs_diff_eq(Quat::IDENTITY, 1e-6));
}

// ─── move_obj tests ─────────────────────────────────────────────────────────

#[test]
fn move_obj_x_translation() {
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
fn move_obj_y_translation() {
    let mut pr = PositionRot::new();
    let ev = MotionEvent {
        x: 0,
        y: 1000,
        z: 0,
        rx: 0,
        ry: 0,
        rz: 0,
        period: 0,
    };
    pr.move_obj(&ev);
    assert_eq!(pr.pos.x, 0.0);
    assert!((pr.pos.y - 1.0).abs() < 1e-6);
    assert_eq!(pr.pos.z, 0.0);
}

#[test]
fn move_obj_z_translation_is_negated() {
    let mut pr = PositionRot::new();
    let ev = MotionEvent {
        x: 0,
        y: 0,
        z: 1000,
        rx: 0,
        ry: 0,
        rz: 0,
        period: 0,
    };
    pr.move_obj(&ev);
    assert_eq!(pr.pos.x, 0.0);
    assert_eq!(pr.pos.y, 0.0);
    assert!((pr.pos.z - (-1.0)).abs() < 1e-6);
}

#[test]
fn move_obj_accumulates_multiple_inputs() {
    let mut pr = PositionRot::new();
    let ev = MotionEvent {
        x: 500,
        y: 500,
        z: 0,
        rx: 0,
        ry: 0,
        rz: 0,
        period: 0,
    };
    pr.move_obj(&ev);
    pr.move_obj(&ev);
    assert!((pr.pos.x - 1.0).abs() < 1e-6);
    assert!((pr.pos.y - 1.0).abs() < 1e-6);
}

#[test]
fn move_obj_negative_translation() {
    let mut pr = PositionRot::new();
    let ev = MotionEvent {
        x: -1000,
        y: -500,
        z: 0,
        rx: 0,
        ry: 0,
        rz: 0,
        period: 0,
    };
    pr.move_obj(&ev);
    assert!((pr.pos.x - (-1.0)).abs() < 1e-6);
    assert!((pr.pos.y - (-0.5)).abs() < 1e-6);
}

#[test]
fn move_obj_pure_rotation() {
    let mut pr = PositionRot::new();
    let ev = MotionEvent {
        x: 0,
        y: 0,
        z: 0,
        rx: 100,
        ry: 0,
        rz: 0,
        period: 0,
    };
    pr.move_obj(&ev);
    assert_eq!(pr.pos, Vec3::ZERO);
    assert!(!pr.rot.abs_diff_eq(Quat::IDENTITY, 1e-6));
}

#[test]
fn move_obj_rotation_around_z() {
    let mut pr = PositionRot::new();
    let ev = MotionEvent {
        x: 0,
        y: 0,
        z: 0,
        rx: 0,
        ry: 0,
        rz: 100,
        period: 0,
    };
    pr.move_obj(&ev);
    assert_eq!(pr.pos, Vec3::ZERO);
    assert!(!pr.rot.abs_diff_eq(Quat::IDENTITY, 1e-6));
}

// ─── move_view tests ────────────────────────────────────────────────────────

#[test]
fn move_view_pure_translation() {
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
    pr.move_view(&ev);
    // In view mode, translation is negated and multiplied by rot
    // With identity rotation, x becomes -1.0
    assert!((pr.pos.x - (-1.0)).abs() < 1e-6);
    assert_eq!(pr.pos.y, 0.0);
    assert_eq!(pr.pos.z, 0.0);
}

#[test]
fn move_view_pure_rotation() {
    let mut pr = PositionRot::new();
    let ev = MotionEvent {
        x: 0,
        y: 0,
        z: 0,
        rx: 100,
        ry: 0,
        rz: 0,
        period: 0,
    };
    pr.move_view(&ev);
    // Position should remain at origin for pure rotation
    assert_eq!(pr.pos, Vec3::ZERO);
    assert!(!pr.rot.abs_diff_eq(Quat::IDENTITY, 1e-6));
}

#[test]
fn move_view_rotates_around_origin() {
    let mut pr = PositionRot::new();
    let ev = MotionEvent {
        x: 0,
        y: 0,
        z: 0,
        rx: 100,
        ry: 0,
        rz: 0,
        period: 0,
    };
    pr.move_view(&ev);
    assert_eq!(pr.pos, Vec3::ZERO);
    assert!(!pr.rot.abs_diff_eq(Quat::IDENTITY, 1e-6));
}

// ─── Matrix tests ───────────────────────────────────────────────────────────

#[test]
fn model_matrix_identity() {
    let pr = PositionRot::new();
    let mat = pr.to_model_matrix();
    // Identity position/rotation should give identity matrix
    assert!((mat.x_axis.x - 1.0).abs() < 1e-6);
    assert!((mat.y_axis.y - 1.0).abs() < 1e-6);
    assert!((mat.z_axis.z - 1.0).abs() < 1e-6);
    assert!((mat.w_axis.x - 0.0).abs() < 1e-6);
    assert!((mat.w_axis.y - 0.0).abs() < 1e-6);
    assert!((mat.w_axis.z - 0.0).abs() < 1e-6);
}

#[test]
fn model_matrix_translation_only() {
    let mut pr = PositionRot::new();
    pr.pos = Vec3::new(1.0, 2.0, 3.0);
    let mat = pr.to_model_matrix();
    // Translation in column-major OpenGL format
    assert!((mat.w_axis.x - 1.0).abs() < 1e-6);
    assert!((mat.w_axis.y - 2.0).abs() < 1e-6);
    assert!((mat.w_axis.z - 3.0).abs() < 1e-6);
}

#[test]
fn view_matrix_identity() {
    let pr = PositionRot::new();
    let mat = pr.to_view_matrix();
    // Identity position/rotation should give identity matrix
    assert!((mat.x_axis.x - 1.0).abs() < 1e-6);
    assert!((mat.y_axis.y - 1.0).abs() < 1e-6);
    assert!((mat.z_axis.z - 1.0).abs() < 1e-6);
    assert!((mat.w_axis.x - 0.0).abs() < 1e-6);
    assert!((mat.w_axis.y - 0.0).abs() < 1e-6);
    assert!((mat.w_axis.z - 0.0).abs() < 1e-6);
}

#[test]
fn view_matrix_translation_only() {
    let mut pr = PositionRot::new();
    pr.pos = Vec3::new(1.0, 2.0, 3.0);
    let mat = pr.to_view_matrix();
    // Translation in column-major OpenGL format
    assert!((mat.w_axis.x - 1.0).abs() < 1e-6);
    assert!((mat.w_axis.y - 2.0).abs() < 1e-6);
    assert!((mat.w_axis.z - 3.0).abs() < 1e-6);
}

#[test]
fn model_and_view_matrices_differ_with_rotation() {
    let mut pr = PositionRot::new();
    pr.pos = Vec3::new(1.0, 2.0, 3.0);
    pr.rot = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
    let model = pr.to_model_matrix();
    let view = pr.to_view_matrix();
    // Model and view should differ when rotation is applied
    let diff = (model.w_axis.x - view.w_axis.x).abs()
        + (model.w_axis.y - view.w_axis.y).abs()
        + (model.w_axis.z - view.w_axis.z).abs();
    assert!(diff > 0.1);
}

// ─── Scaling factor tests ───────────────────────────────────────────────────

#[test]
fn move_obj_scaling_factor_is_0001() {
    let mut pr = PositionRot::new();
    let ev = MotionEvent {
        x: 100,
        y: 200,
        z: 300,
        rx: 0,
        ry: 0,
        rz: 0,
        period: 0,
    };
    pr.move_obj(&ev);
    assert!((pr.pos.x - 0.1).abs() < 1e-6);
    assert!((pr.pos.y - 0.2).abs() < 1e-6);
    assert!((pr.pos.z - (-0.3)).abs() < 1e-6);
}
