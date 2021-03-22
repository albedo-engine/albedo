use gltf::Camera;

#[enumflags2::bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CameraMoveCommand {
    None = 0b0000_0001,
    Forward,
    Backward,
    Left,
    Right = 0b0001_0000,
}

#[derive(Default)]
pub struct CameraController {
    pub move_speed_factor: f32,
    pub move_velocity: glam::Vec3,
    pub rot_velocity: glam::Vec2,
    pub rot_speed_factor: glam::Vec2,
    pub move_damping_factor: f32,
    pub rot_damping_factor: f32,
    pub origin: glam::Vec3,
    pub direction: glam::Vec3,
    pub commands: enumflags2::BitFlags<CameraMoveCommand>,
}

impl CameraController {
    pub fn new() -> Self {
        CameraController {
            move_damping_factor: 0.5,
            move_speed_factor: 1.0,
            rot_damping_factor: 0.5,
            rot_speed_factor: glam::Vec2::new(1.0, 1.0),
            direction: glam::Vec3::new(0.0, 0.0, -1.0),
            ..Default::default()
        }
    }

    pub fn from_origin_dir(origin: glam::Vec3, direction: glam::Vec3) -> Self {
        let mut def = CameraController::new();
        def.origin = origin;
        def.direction = direction;
        return def;
    }

    pub fn rotate(&mut self, x: f32, y: f32) {
        self.rot_velocity.x += x;
        self.rot_velocity.y += y;
    }

    pub fn set_command(&mut self, cmd: CameraMoveCommand) {
        self.commands.insert(enumflags2::BitFlags::from(cmd));
    }

    pub fn unset_command(&mut self, cmd: CameraMoveCommand) {
        self.commands.remove(enumflags2::BitFlags::from(cmd));
    }

    pub fn update(&mut self, delta: f32) -> (glam::Vec3, glam::Vec3) {
        let mut right = self.direction.cross(glam::Vec3::unit_y()).normalize();
        let mut up = right.cross(self.direction).normalize();

        let rot_velocity = self.rot_velocity * self.rot_speed_factor * delta;
        let rot = glam::Quat::from_axis_angle(up, -rot_velocity.x)
            * glam::Quat::from_axis_angle(right, -rot_velocity.y);

        self.direction = (rot * self.direction).normalize();
        right = self.direction.cross(glam::Vec3::unit_y()).normalize();
        up = right.cross(self.direction).normalize();

        if self.commands.contains(CameraMoveCommand::Left) {
            self.move_velocity.x += -1.0;
        }
        if self.commands.contains(CameraMoveCommand::Right) {
            self.move_velocity.x += 1.0;
        }
        if self.commands.contains(CameraMoveCommand::Forward) {
            self.move_velocity.z += 1.0;
        }
        if self.commands.contains(CameraMoveCommand::Backward) {
            self.move_velocity.z += -1.0;
        }
        let move_velocity = self.move_velocity * self.move_speed_factor * delta;

        let move_force = right * move_velocity.x + self.direction * move_velocity.z;
        self.origin += move_force;

        let move_damping = (1.0 - self.move_damping_factor).clamp(0.0, 1.0);
        let rot_damping = (1.0 - self.rot_damping_factor).clamp(0.0, 1.0);

        self.rot_velocity = self.rot_velocity * rot_damping;
        self.move_velocity = self.move_velocity * move_damping;

        return (right, up);
    }

    pub fn is_static(&self) -> bool {
        self.rot_velocity.length_squared() < 0.00000001
            && self.move_velocity.length_squared() < 0.00000001
    }
}
