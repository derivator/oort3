mod user;

pub mod sys {
    use oort_shared::SystemState;

    #[no_mangle]
    pub static mut SYSTEM_STATE: [f64; SystemState::Size as usize] =
        [0.0; SystemState::Size as usize];

    pub fn read_system_state(index: SystemState) -> f64 {
        unsafe { SYSTEM_STATE[index as usize] }
    }

    pub fn write_system_state(index: SystemState, value: f64) {
        unsafe {
            SYSTEM_STATE[index as usize] = value;
        }
    }
}

pub mod vec {
    pub type Vec2 = nalgebra::Vector2<f64>;

    pub trait Vec2Extras {
        fn angle0(&self) -> f64;
        fn distance(&self, other: Vec2) -> f64;
        fn rotate(&self, angle: f64) -> Vec2;
    }

    impl Vec2Extras for Vec2 {
        fn distance(&self, other: Vec2) -> f64 {
            self.metric_distance(&other)
        }

        fn angle0(&self) -> f64 {
            let mut a = self.y.atan2(self.x);
            if a < 0.0 {
                a += std::f64::consts::TAU;
            }
            a
        }

        fn rotate(&self, angle: f64) -> Vec2 {
            nalgebra::Rotation2::new(angle).transform_vector(self)
        }
    }

    pub fn vec2(x: f64, y: f64) -> Vec2 {
        Vec2::new(x, y)
    }
}

pub mod math {
    use std::f64::consts::{PI, TAU};

    pub fn normalize_angle(a: f64) -> f64 {
        let mut a = a;
        if a.abs() > TAU {
            a %= TAU;
        }
        if a < 0.0 {
            a += TAU;
        }
        a
    }

    pub fn angle_diff(a: f64, b: f64) -> f64 {
        let c = normalize_angle(b - a);
        if c > PI {
            c - TAU
        } else {
            c
        }
    }
}

pub mod api {
    use super::sys::{read_system_state, write_system_state};
    use super::vec::*;
    use oort_shared::{Class, SystemState};

    pub fn class() -> Class {
        Class::from_f64(read_system_state(SystemState::Class))
    }

    pub fn seed() -> u128 {
        read_system_state(oort_shared::SystemState::Seed) as u128
    }

    pub fn position() -> Vec2 {
        vec2(
            read_system_state(SystemState::PositionX),
            read_system_state(SystemState::PositionY),
        )
    }

    pub fn velocity() -> Vec2 {
        vec2(
            read_system_state(SystemState::VelocityX),
            read_system_state(SystemState::VelocityY),
        )
    }

    pub fn heading() -> f64 {
        read_system_state(SystemState::Heading)
    }

    pub fn angular_velocity() -> f64 {
        read_system_state(SystemState::AngularVelocity)
    }

    pub fn accelerate(acceleration: Vec2) {
        write_system_state(SystemState::AccelerateX, acceleration.x);
        write_system_state(SystemState::AccelerateY, acceleration.y);
    }

    pub fn torque(angular_acceleration: f64) {
        write_system_state(SystemState::Torque, angular_acceleration);
    }

    pub fn aim_gun(gun_index: usize, heading: f64) {
        let state_index = match gun_index {
            0 => SystemState::Gun0Aim,
            1 => SystemState::Gun1Aim,
            2 => SystemState::Gun2Aim,
            3 => SystemState::Gun3Aim,
            _ => return,
        };
        write_system_state(state_index, heading);
    }

    pub fn fire_gun(gun_index: usize) {
        let state_index = match gun_index {
            0 => SystemState::Gun0Fire,
            1 => SystemState::Gun1Fire,
            2 => SystemState::Gun2Fire,
            3 => SystemState::Gun3Fire,
            _ => return,
        };
        write_system_state(state_index, 1.0);
    }

    pub fn launch_missile(missile_index: usize, _orders: &str) {
        let state_index = match missile_index {
            0 => SystemState::Missile0Launch,
            1 => SystemState::Missile1Launch,
            2 => SystemState::Missile2Launch,
            3 => SystemState::Missile3Launch,
            _ => return,
        };
        write_system_state(state_index, 1.0);
    }

    pub fn explode() {
        write_system_state(SystemState::Explode, 1.0);
    }

    pub fn set_radar_heading(heading: f64) {
        write_system_state(SystemState::RadarHeading, heading);
    }

    pub fn set_radar_width(width: f64) {
        write_system_state(SystemState::RadarWidth, width);
    }

    pub struct ScanResult {
        pub class: Class,
        pub position: Vec2,
        pub velocity: Vec2,
    }

    pub fn scan() -> Option<ScanResult> {
        if read_system_state(SystemState::RadarContactFound) == 0.0 {
            return None;
        }
        Some(ScanResult {
            class: Class::from_f64(read_system_state(SystemState::RadarContactClass)),
            position: vec2(
                read_system_state(SystemState::RadarContactPositionX),
                read_system_state(SystemState::RadarContactPositionY),
            ),
            velocity: vec2(
                read_system_state(SystemState::RadarContactVelocityX),
                read_system_state(SystemState::RadarContactVelocityY),
            ),
        })
    }
}

pub mod prelude {
    pub use super::api::*;
    pub use super::math::*;
    pub use super::vec::*;
    pub use oort_shared::*;
}

static mut USER_STATE: Option<user::Ship> = None;

#[no_mangle]
pub fn export_tick() {
    unsafe {
        if USER_STATE.is_none() {
            USER_STATE = Some(user::Ship::new());
        }
        USER_STATE.as_mut().unwrap().tick();
    }
}