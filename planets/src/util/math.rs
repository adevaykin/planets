use cgmath as cgm;

pub fn direction_to_rotation(
    direction: &cgm::Vector3<f32>,
    rotation_axis: &cgm::Vector3<f32>,
) -> cgm::Matrix4<f32> {
    let right = rotation_axis.cross(*direction);
    let up = right.cross(*direction);
    let col1 = cgm::Vector3::new(right.x, direction.x, up.x);
    let col2 = cgm::Vector3::new(right.y, direction.y, up.y);
    let col3 = cgm::Vector3::new(right.z, direction.z, up.z);
    let rotation = cgm::Matrix3::from_cols(col1, col2, col3);

    cgm::Matrix4::from(rotation)
}

pub fn position_from_transform(transform: &cgm::Matrix4<f32>) -> cgm::Vector3<f32> {
    cgm::Vector3::new(transform[3].x, transform[3].y, transform[3].z)
}

pub fn set_translation(transform: &mut cgm::Matrix4<f32>, translation: &cgm::Vector3<f32>) {
    transform[3] = cgm::Vector4::new(translation.x, translation.y, translation.z, transform[3].w);
}

pub fn teleport_transform_to_screen(transform: &mut cgm::Matrix4<f32>, aspect_ratio: f32) {
    let mut position = position_from_transform(transform);
    teleport_position_to_screen(&mut position, aspect_ratio);
    set_translation(transform, &position)
}

pub fn teleport_position_to_screen(position: &mut cgm::Vector3<f32>, aspect_ratio: f32) {
    const SCREEN_END_Y: f32 = 4.8;
    let screen_end_x = SCREEN_END_Y * aspect_ratio;

    if position.x.abs() > screen_end_x {
        position.x = -position.x;
    }
    if position.y.abs() > SCREEN_END_Y {
        position.y = -position.y;
    }
}
