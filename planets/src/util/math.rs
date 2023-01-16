use cgmath as cgm;

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn set_translation(transform: &mut cgm::Matrix4<f32>, translation: &cgm::Vector3<f32>) {
    transform[3] = cgm::Vector4::new(translation.x, translation.y, translation.z, transform[3].w);
}
