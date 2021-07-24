use boxcars::Quaternion;

pub fn euler_from_quat(quaternion: Quaternion) -> (f32, f32, f32) {
    // https://en.wikipedia.org/wiki/Conversion_between_quaternions_and_Euler_angles#Quaternion_to_Euler_angles_conversion
    let w = quaternion.w;
    let y = quaternion.y;
    let x = quaternion.x;
    let z = quaternion.z;

    let sinr = 2.0 * (w * x + y * z);
    let cosr = 1.0 - 2.0 * (x * x + y * y);
    let roll = sinr.atan2(cosr);

    let sinp = 2.0 * (w * y - z * x);
    let pitch: f32;
    if sinp.abs() >= 1.0 {
        pitch = (std::f32::consts::PI / 2.0).copysign(sinp);
    } else {
        pitch = sinp.asin();
    }

    let siny = 2.0 * (w * z + x * y);
    let cosy = 1.0 - 2.0 * (y * y + z * z);
    let yaw = siny.atan2(cosy);
    (pitch, yaw, roll)
}
