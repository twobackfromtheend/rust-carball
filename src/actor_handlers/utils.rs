use crate::frame_parser::Actor;
use boxcars::attributes::{RemoteId, UniqueId};
use boxcars::Attribute;
use log::warn;
use serde::{Serialize, Serializer};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::fmt;
use std::hash::{Hash, Hasher};

pub struct RigidBodyData {
    pub is_sleeping: Option<bool>,
    pub pos_x: Option<f32>,
    pub pos_y: Option<f32>,
    pub pos_z: Option<f32>,
    pub vel_x: Option<f32>,
    pub vel_y: Option<f32>,
    pub vel_z: Option<f32>,
    pub quat_w: Option<f32>,
    pub quat_x: Option<f32>,
    pub quat_y: Option<f32>,
    pub quat_z: Option<f32>,
    pub ang_vel_x: Option<f32>,
    pub ang_vel_y: Option<f32>,
    pub ang_vel_z: Option<f32>,
}

impl RigidBodyData {
    pub fn from(
        actor: &Actor,
        attributes: &HashMap<String, boxcars::Attribute>,
        replay_version: i32,
    ) -> Self {
        if replay_version < 2 {
            panic!("Cannot parse replay version < 2")
        }
        let initial_location = actor
            .new_actor
            .initial_trajectory
            .location
            .expect("RB actor has no initial location.");

        let mut is_sleeping = None;
        let mut pos_x = Some(initial_location.x as f32);
        let mut pos_y = Some(initial_location.y as f32);
        let mut pos_z = Some(initial_location.z as f32);
        let mut vel_x = None;
        let mut vel_y = None;
        let mut vel_z = None;
        // TODO: Find out how initial_rotation should be used.
        // let initial_rotation = actor
        //     .new_actor
        //     .initial_trajectory
        //     .rotation
        //     .expect("Car actor has no initial rotation.");
        // let mut rot_pitch = initial_rotation.pitch.map(|rot| rot as f32);
        // let mut rot_yaw = initial_rotation.yaw.map(|rot| rot as f32);
        // let mut rot_roll = initial_rotation.roll.map(|rot| rot as f32);
        let mut quat_w = None;
        let mut quat_x = None;
        let mut quat_y = None;
        let mut quat_z = None;
        let mut ang_vel_x = None;
        let mut ang_vel_y = None;
        let mut ang_vel_z = None;

        if let Some(Attribute::RigidBody(rb_state)) =
            attributes.get("TAGame.RBActor_TA:ReplicatedRBState")
        {
            is_sleeping = Some(rb_state.sleeping);

            let location = rb_state.location;
            if replay_version >= 7 {
                pos_x = Some(location.x);
                pos_y = Some(location.y);
                pos_z = Some(location.z);
            } else if replay_version >= 2 {
                pos_x = Some(location.x * 100.0);
                pos_y = Some(location.y * 100.0);
                pos_z = Some(location.z * 100.0);
            }
            if let Some(linear_velocity) = rb_state.linear_velocity {
                if let Some(angular_velocity) = rb_state.angular_velocity {
                    if replay_version >= 7 {
                        vel_x = Some(linear_velocity.x);
                        vel_y = Some(linear_velocity.y);
                        vel_z = Some(linear_velocity.z);
                    } else if replay_version >= 2 {
                        vel_x = Some(linear_velocity.x * 10.0);
                        vel_y = Some(linear_velocity.y * 10.0);
                        vel_z = Some(linear_velocity.z * 10.0);
                    }
                    if replay_version >= 8 {
                        quat_w = Some(rb_state.rotation.w);
                        quat_x = Some(rb_state.rotation.x);
                        quat_y = Some(rb_state.rotation.y);
                        quat_z = Some(rb_state.rotation.z);
                    } else {
                        if rb_state.rotation.w != 0.0 {
                            warn!(
                                "non-zero w for rotation for replay version {}",
                                replay_version
                            )
                        }
                        let pitch = rb_state.rotation.x;
                        let yaw = rb_state.rotation.y;
                        let roll = rb_state.rotation.z;
                        let quat = rotator_to_quat(pitch, yaw, roll);

                        quat_w = Some(quat.0);
                        quat_x = Some(quat.1);
                        quat_y = Some(quat.2);
                        quat_z = Some(quat.3);
                    }

                    // Dividing by 100 to result in radians/s
                    ang_vel_x = Some(angular_velocity.x / 100.0);
                    ang_vel_y = Some(angular_velocity.y / 100.0);
                    ang_vel_z = Some(angular_velocity.z / 100.0);
                }
            }
        }

        Self {
            is_sleeping,
            pos_x,
            pos_y,
            pos_z,
            vel_x,
            vel_y,
            vel_z,
            quat_w,
            quat_x,
            quat_y,
            quat_z,
            ang_vel_x,
            ang_vel_y,
            ang_vel_z,
        }
    }
}

/// Converts euler angles from game into quaternion (w, x, y, z).
fn rotator_to_quat(pitch: f32, yaw: f32, roll: f32) -> (f32, f32, f32, f32) {
    let sin_pitch = f32::sin(pitch / 2.0);
    let cos_pitch = f32::cos(pitch / 2.0);
    let sin_yaw = f32::sin(yaw / 2.0);
    let cos_yaw = f32::cos(yaw / 2.0);
    let sin_roll = f32::sin(roll / 2.0);
    let cos_roll = f32::cos(roll / 2.0);

    let w = (cos_roll * cos_pitch * cos_yaw) + (sin_roll * sin_pitch * sin_yaw);
    let x = (sin_roll * cos_pitch * cos_yaw) - (cos_roll * sin_pitch * sin_yaw);
    let y = (cos_roll * sin_pitch * cos_yaw) + (sin_roll * cos_pitch * sin_yaw);
    let z = (cos_roll * cos_pitch * sin_yaw) - (sin_roll * sin_pitch * cos_yaw);

    let norm = f32::sqrt(w * w + x * x + y * y + z * z);
    (w / norm, x / norm, y / norm, z / norm)
}

#[derive(Debug, Clone)]
pub struct WrappedUniqueId(UniqueId);

impl WrappedUniqueId {
    pub fn from(attributes: &HashMap<String, Attribute>) -> Self {
        if let Some(Attribute::UniqueId(unique_id)) =
            attributes.get("Engine.PlayerReplicationInfo:UniqueId")
        {
            Self(unique_id.as_ref().clone())
        } else {
            panic!("Could not get UniqueId attribute.")
        }
    }
}

impl Hash for WrappedUniqueId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.0.remote_id {
            RemoteId::PlayStation(ps4_id) => {
                "PlayStation".hash(state);
                ps4_id.online_id.hash(state);
                ps4_id.name.hash(state);
            }
            RemoteId::PsyNet(psy_net_id) => {
                "PsyNet".hash(state);
                psy_net_id.online_id.hash(state);
            }
            RemoteId::SplitScreen(i) => {
                "SplitScreen".hash(state);
                i.hash(state);
            }
            RemoteId::Steam(i) => {
                "Steam".hash(state);
                i.hash(state);
            }
            RemoteId::Switch(switch_id) => {
                "Switch".hash(state);
                switch_id.online_id.hash(state);
            }
            RemoteId::Xbox(i) => {
                "Xbox".hash(state);
                i.hash(state);
            }
            RemoteId::QQ(i) => {
                "QQ".hash(state);
                i.hash(state);
            }
            RemoteId::Epic(string) => {
                "Epic".hash(state);
                string.hash(state);
            }
        }
    }
}

impl PartialEq for WrappedUniqueId {
    fn eq(&self, other: &WrappedUniqueId) -> bool {
        // TODO: Replace with accurate impl (referencing hash impl).
        self.0.remote_id == other.0.remote_id
    }
}
impl Eq for WrappedUniqueId {}

impl fmt::Display for WrappedUniqueId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        write!(f, "{}", hasher.finish())
    }
}

impl Serialize for WrappedUniqueId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
