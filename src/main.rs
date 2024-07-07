pub mod macros;
pub mod simple_simulator;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use macros::{on_key_pressed_force_set, on_key_pressed_impulse};
use simple_simulator::{build_standard_in_engine, control_player_rocket, follow_ship_camera, render_forces};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, setup_physics)
        .add_systems(Startup, add_an_arena)
        .add_systems(Startup, build_standard_in_engine)
        .add_systems(Update, (control_player_rocket, follow_ship_camera, render_forces))
        .run();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 12.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn setup_physics(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    /* Create the ground. */
    commands
        .spawn(Collider::cuboid(100.0, 0.1, 100.0))
        .insert(PbrBundle {
            mesh: meshes.add(Cuboid::new(100.0, 0.1, 100.0)),
            material: materials.add(Color::GRAY),
            transform: Transform::from_xyz(0.0, -2.0, 0.0),
            ..default()
        });

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(5.0, 0.1, 0.1)),
            material: materials.add(Color::RED),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(0.1, 0.1, 5.0)),
            material: materials.add(Color::BLUE),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });    


    /* Create the bouncing ball. */
    commands
        .spawn(RigidBody::Dynamic)
        .insert(Player)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7))
        .insert(Friction::coefficient(0.2))
        .insert(ExternalImpulse {
            impulse: Vec3::ZERO,
            torque_impulse: Vec3::ZERO,
        })
        .insert(ExternalForce {
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
        })
        .insert(PbrBundle {
            mesh: meshes.add(Sphere::new(0.5)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(0.0, 4.0, 0.0),
            ..default()
        });
}

fn control_player(mut player: Query<(&mut ExternalImpulse, &mut ExternalForce), With<Player>>, keys: Res<ButtonInput<KeyCode>>) {
    for (mut impulse, mut force) in player.iter_mut() {
        
        on_key_pressed_force_set!(keys, KeyCode::KeyW, force, Vec3::Z * -0.5);
        on_key_pressed_force_set!(keys, KeyCode::KeyS, force, Vec3::Z * 0.5);
        on_key_pressed_force_set!(keys, KeyCode::KeyA, force, Vec3::X * -0.5);
        on_key_pressed_force_set!(keys, KeyCode::KeyD, force, Vec3::X * 0.5);
        
        on_key_pressed_impulse!(keys, KeyCode::Space, impulse, Vec3::Y * 3.0);
    }
}

fn add_an_arena(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {

    macro_rules! add_a_cuboid {
        ($hx:expr,$hy:expr,$hz:expr,$position:expr,$color:expr) => {
          commands
            .spawn(RigidBody::Fixed) 
            .insert(Collider::cuboid($hx / 2.0, $hy / 2.0, $hz / 2.0))
            .insert(PbrBundle {
              mesh: meshes.add(Cuboid::new($hx, $hy, $hz)),
              material: materials.add(Color::BLACK),
              transform: Transform::from_translation($position),
              ..default()
            });     
        };
    }

    add_a_cuboid!(1.0, 2.0, 10.0, Vec3::new(-5.0, 0.0, 0.0), Color::BLACK);
    add_a_cuboid!(1.0, 2.0, 10.0, Vec3::new(5.0, 0.0, 0.0), Color::BLACK);
    add_a_cuboid!(10.0, 2.0, 1.0, Vec3::new(0.0, 0.0, 5.0), Color::YELLOW);
    add_a_cuboid!(10.0, 2.0, 1.0, Vec3::new(0.0, 0.0, -5.0), Color::YELLOW);
}

fn follow_player(player: Query<&Transform, With<Player>>, mut camera: Query<&mut Transform, (With<Camera>, Without<Player>)>) {
    for player_position in &player {
        for mut camera_position in camera.iter_mut() {
            *camera_position =  Transform::from_translation(player_position.translation + Vec3::new(0.0, 12.0, 8.0)).looking_at(player_position.translation, Vec3::Y);
        }
    }
}



#[derive(Component)]
pub struct Player;
