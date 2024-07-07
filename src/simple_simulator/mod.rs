use bevy::{prelude::*, transform::helper};
use bevy_rapier3d::{parry::transformation::utils::transform, prelude::*};

#[derive(Clone)]
pub struct Engine{
    pub isp: f32,
    pub thrust: f32
}

#[derive(Clone, Copy)]
pub enum BodyShape {
    Cyllinder {
        half_height: f32,
        radius: f32
    },
    Capsule {
        half_height: f32,
        radius: f32
    }
}

#[derive(Clone)]
pub enum ControlType {
    PitchUp,
    PitchDown,
    YawLeft,
    YawRight,
    Accelarete
}

#[derive(Clone, Component)]
pub struct EngineLocation {
    offset: Vec3,
    control_type: ControlType,
    engine: Engine
}

#[derive(Component, Resource)]
pub struct Ship {
    pub shape: BodyShape,
    pub density: f32,
    pub drive: EngineLocation,
    pub thrusters: Vec<EngineLocation>
}

#[derive(Component, Default)]
pub struct AbstractForce {
    pub global_point: Vec3,
    pub global_vector: Vec3
}

impl AbstractForce {

    pub fn to_external_force(&self, center_of_mass: Vec3) -> ExternalForce {
        ExternalForce::at_point(self.global_vector, self.global_point, center_of_mass)
    }
    
}

pub fn build_standard_in_engine(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let drive = Engine {
        isp: 10000.0,
        thrust: 8e3
    };

    let direction_thruster = Engine {
        isp: 300.0,
        thrust: 200.0
    };

    macro_rules! add_thruster {
        ($offset:expr, $control_type:expr) => {
            EngineLocation {
                offset: $offset,
                control_type: $control_type,
                engine: direction_thruster.clone()
            }
        };
    }

    let definition = Ship {
        shape: BodyShape::Cyllinder { half_height: 5.0, radius: 1.0 },
        density: 10.0,
        drive: EngineLocation {
            offset: Vec3::NEG_Y * 5.0,
            control_type: ControlType::Accelarete,
            engine: drive
        },
        thrusters: vec![
            add_thruster!(Vec3::NEG_X + Vec3::Y * 2.0, ControlType::YawRight),
            add_thruster!(Vec3::X + Vec3::Y * 2.0, ControlType::YawLeft),
            add_thruster!(Vec3::NEG_Z + Vec3::Y * 2.0, ControlType::PitchDown),
            add_thruster!(Vec3::Z + Vec3::Y * 2.0, ControlType::PitchUp)
        ]
    }; 

    let shape = definition.shape;
    let engines = definition.thrusters.clone();

    let rendered_engines = render_engines(&engines, &mut meshes, &mut materials);
    let forces_of_engines = create_force_entries(&engines);

    let rendered_drive = PbrBundle {
        mesh: meshes.add(bevy::prelude::Cylinder { radius: 0.5, half_height: 0.7 }),
        material: materials.add(Color::YELLOW),
        transform: Transform::from_translation(definition.drive.offset),
        ..Default::default()
    };

    let thrusters_bundle: Vec<_> = engines.into_iter().zip(rendered_engines).zip(forces_of_engines).collect();

    commands
      .spawn(definition)      
      .insert(RigidBody::Dynamic)
      .insert(get_collider(shape))
      .insert(ColliderMassProperties::Density(10.0))
      .insert(main_body_mesh(shape, Transform::from_translation(Vec3::Y * 20.0), &mut meshes, &mut materials))
      .insert(ExternalForce {
        force: Vec3::ZERO,
        torque: Vec3::ZERO,
      })
      .insert(AbstractForce::default())
      .with_children(|parent| {
        parent.spawn(rendered_drive);
        for ((engine, rendered_engine), force) in thrusters_bundle {
            parent
              .spawn(engine)
              .insert(rendered_engine)
              .insert(force);
        }
      });
        

}

fn main_body_mesh(
    shape: BodyShape,
    location: Transform,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>
) -> PbrBundle {
    let mesh: Mesh = match shape {
        BodyShape::Cyllinder { half_height, radius } => bevy::prelude::Cylinder { radius, half_height }.into(),
        BodyShape::Capsule { half_height, radius } => bevy::prelude::Capsule3d { radius, half_length: half_height }.into(),
    };
    PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::GRAY),
        transform: location,
        ..Default::default()
    }
}

fn render_engines(
    engines: &Vec<EngineLocation>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials:&mut ResMut<Assets<StandardMaterial>>
) -> Vec<PbrBundle> {
    let mesh = bevy::prelude::Cylinder { radius: 0.3, half_height: 0.5 };
    
    engines
      .iter()
      .map(|engine| {
        let normal_vector = engine.offset.reject_from(Vec3::Y);
        println!("Got angle for engine: {}", Vec3::Y.angle_between(normal_vector));
        let rotation = Quat::from_rotation_arc(Vec3::Y, normal_vector);
        PbrBundle {
            mesh: meshes.add(mesh.clone()),
            material: materials.add(Color::RED),
            transform: Transform::from_translation(engine.offset).with_rotation(rotation),
            ..Default::default()
        }
      })
      .collect()
}

fn create_force_entries(
    engines: &Vec<EngineLocation>,
) -> Vec<AbstractForce> {
    engines
      .iter()
      .map(|_| AbstractForce::default())
      .collect()
}


fn get_collider(shape: BodyShape) -> Collider {
    match shape {
        BodyShape::Cyllinder { half_height, radius } => {
            Collider::cylinder(half_height, radius)
        },
        BodyShape::Capsule { half_height, radius } => {
            Collider::capsule_z(half_height, radius)
        },
    }
}

//ship dynamic handlers
pub fn control_player_rocket(
    keys: Res<ButtonInput<KeyCode>>,
    mut ship: Query<(&Ship, &Transform, &mut ExternalForce, &mut AbstractForce, &Children), Without<EngineLocation>>,
    mut direction_thruster: Query<(&EngineLocation, &mut AbstractForce)>
) {
    for (ship, transform, mut force, mut drive_force, children) in ship.iter_mut() {
        let force_direction = transform.local_y();
        if keys.pressed(KeyCode::Space) {
            drive_force.global_vector = force_direction * ship.drive.engine.thrust;
        } else {
            drive_force.global_vector = Vec3::ZERO;
        }

        let mut final_force = ExternalForce { force: drive_force.global_vector, torque: Vec3::ZERO };

        for child in children {
            if let Ok((engine, mut force)) = direction_thruster.get_mut(*child) {
                macro_rules! apply_force {
                    ($force_direction:ident) => {
                        let force_location = transform.translation + transform.rotation * engine.offset;
                        *force = AbstractForce { global_point: force_location, global_vector: $force_direction * engine.engine.thrust };
                    };
                }
                match &engine.control_type {
                    ControlType::PitchUp => {
                        let force_direction = transform.local_z() * -1.0;
                        if keys.pressed(KeyCode::KeyW) {
                            apply_force!(force_direction);
                        } else {
                            *force = AbstractForce::default();
                        }
                        final_force += force.to_external_force(transform.translation);
                    },
                    ControlType::PitchDown => {
                        let force_direction = transform.local_z();
                        if keys.pressed(KeyCode::KeyS) {
                            apply_force!(force_direction);
                        } else {
                            *force = AbstractForce::default();
                        }
                        final_force += force.to_external_force(transform.translation);
                    },
                    ControlType::YawLeft => {
                        let force_direction = transform.local_x() * -1.0;
                        if keys.pressed(KeyCode::KeyA) {
                            apply_force!(force_direction);
                        } else {
                            *force = AbstractForce::default();
                        }
                        final_force += force.to_external_force(transform.translation);
                    },
                    ControlType::YawRight => {
                        let force_direction = transform.local_x();
                        if keys.pressed(KeyCode::KeyD) {
                            apply_force!(force_direction);
                        } else {
                            *force = AbstractForce::default();
                        }
                        final_force += force.to_external_force(transform.translation);
                    },
                    ControlType::Accelarete => {},
                }
            }
        }    
        *force = final_force;    
    }
}

pub fn follow_ship_camera(player: Query<&Transform, With<Ship>>, mut camera: Query<&mut Transform, (With<Camera>, Without<Ship>)>) {
    for player_position in &player {
        for mut camera_position in camera.iter_mut() {
            *camera_position =  Transform::from_translation(player_position.translation + Vec3::new(0.0, 12.0, 8.0)).looking_at(player_position.translation, Vec3::Y);
        }
    }
}

pub fn render_forces(
     mut gizmos: Gizmos,
     ship: Query<(Entity, &Ship, &Children)>,
     get_transform_and_force: Query<(&Transform, &AbstractForce), Without<EngineLocation>>,
     get_engine_and_force: Query<&AbstractForce>
    ) {
    for (id, ship, children) in ship.iter() {
        if let Ok((transform, force)) = get_transform_and_force.get(id) {
            let helper = match ship.shape {
                BodyShape::Cyllinder { half_height, .. } => half_height,
                BodyShape::Capsule { half_height, radius } => half_height + radius,
            };
            let start = transform.translation + transform.local_y() * -1.0 * helper;
            let end = start - force.global_vector * 0.01;
            if end != start {
                gizmos.arrow(start, end, Color::BLUE);
            }


            for child in children {
                if let Ok(force) = get_engine_and_force.get(*child) {
                    let start = force.global_point;
                    let end = start - force.global_vector * 0.01;
                    if end != start {
                        gizmos.arrow(start, end, Color::RED);
                    }
                }
            }
        } else {
            //lama
        }
    }
}
