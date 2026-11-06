use horizont_vr::crossplay::{HandGesture, VrPlayerPosePacket};
use horizont_vr::math::{Mat4, Quat, Vec3};
use horizont_vr::stereo::{compute_eye_view_matrix, EyeFov, StereoEye};
use horizont_vr::tracking::{ControllerState, Pose6DoF, VrTrackingSnapshot};

#[test]
fn test_vr_projection_matrix() {
    let fov = EyeFov::default();
    let proj = fov.to_projection_matrix(0.05, 1000.0);
    // Projection matrix diagonal elements should be non-zero
    assert!(proj.0[0][0] > 0.0);
    assert!(proj.0[1][1] > 0.0);
    assert_ne!(proj.0[2][2], 0.0);
}

#[test]
fn test_vr_stereo_view_matrices() {
    let head_pose = Pose6DoF::new([0.0, 1.62, 0.0], [0.0, 0.0, 0.0, 1.0]);
    let ipd = 0.063; // 63mm IPD

    let view_left = compute_eye_view_matrix([0.0, 0.0, 0.0], &head_pose, StereoEye::Left, ipd);
    let view_right = compute_eye_view_matrix([0.0, 0.0, 0.0], &head_pose, StereoEye::Right, ipd);

    // Left and right eye view matrices must be shifted by IPD along X axis
    assert_ne!(view_left.0[3][0], view_right.0[3][0]);
    // The difference between left and right X translation should match IPD
    let diff = (view_left.0[3][0] - view_right.0[3][0]).abs();
    assert!((diff - ipd).abs() < 1e-4, "IPD offset should match: diff={diff}");
}

#[test]
fn test_vr_crossplay_packet() {
    let head = Pose6DoF::new([10.5, 64.0, -20.2], [0.0, 0.7071, 0.0, 0.7071]);
    let mut left_ctrl = ControllerState::default();
    left_ctrl.grip = 0.8;
    left_ctrl.trigger = 0.9; // Clenched fist

    let mut right_ctrl = ControllerState::default();
    right_ctrl.grip = 0.8;
    right_ctrl.trigger = 0.1; // Pointing finger

    let packet = VrPlayerPosePacket::create_local("test-uuid-1234", &head, &left_ctrl, &right_ctrl);
    assert_eq!(packet.left_gesture, HandGesture::Fist);
    assert_eq!(packet.right_gesture, HandGesture::Point);

    // Serialize to bytes and deserialize
    let bytes = packet.serialize().expect("Serialize should succeed");
    let decoded = VrPlayerPosePacket::deserialize(&bytes).expect("Deserialize should succeed");

    assert_eq!(decoded.player_uuid, "test-uuid-1234");
    assert_eq!(decoded.left_gesture, HandGesture::Fist);
    assert_eq!(decoded.right_gesture, HandGesture::Point);
    assert_eq!(decoded.head_pose.position, [10.5, 64.0, -20.2]);
}

#[test]
fn test_vr_controller_raycast() {
    let mut snap = VrTrackingSnapshot::default();
    snap.right_controller.aim_pose = Pose6DoF::new([0.0, 1.2, 0.0], [0.0, 0.0, 0.0, 1.0]);

    let (origin, dir) = snap.controller_ray(true);
    assert_eq!(origin, [0.0, 1.2, 0.0]);
    // Default forward vector in Minecraft/VR points along negative Z
    assert_eq!(dir, [0.0, 0.0, -1.0]);
}
