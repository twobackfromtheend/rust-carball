use crate::frame_parser::Actor;
use boxcars::attributes::{RemoteId, UniqueId};
use boxcars::{Attribute, Quaternion};
use serde::{Serialize, Serializer};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};

pub struct RigidBodyData {
    pub is_sleeping: Option<bool>,
    pub pos_x: Option<f32>,
    pub pos_y: Option<f32>,
    pub pos_z: Option<f32>,
    pub vel_x: Option<f32>,
    pub vel_y: Option<f32>,
    pub vel_z: Option<f32>,
    pub rot_pitch: Option<f32>,
    pub rot_yaw: Option<f32>,
    pub rot_roll: Option<f32>,
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
        let mut rot_pitch = None;
        let mut rot_yaw = None;
        let mut rot_roll = None;
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

                    let eulers = euler_from_quat(rb_state.rotation);
                    rot_pitch = Some(eulers.0);
                    rot_yaw = Some(eulers.1);
                    rot_roll = Some(eulers.2);

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
            rot_pitch,
            rot_yaw,
            rot_roll,
            ang_vel_x,
            ang_vel_y,
            ang_vel_z,
        }
    }
}

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

#[derive(Debug, PartialEq, Clone)]
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

impl Eq for WrappedUniqueId {}

impl WrappedUniqueId {
    pub fn to_string(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish().to_string()
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
