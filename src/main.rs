use bevy::{
	pbr::AmbientLight, prelude::*,
	input::{keyboard::*, mouse::*, Input, ElementState},
	render::camera::*,
	diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}
};
use bevy_mod_raycast::{DefaultRaycastingPlugin, RayCastMesh, RayCastMethod, RayCastSource, RaycastSystem, Ray3d};
use heron::prelude::*;

//Resources
#[derive(Default)]
struct PlayerRayDir(Vec3); 

struct MyRaycastSet;

struct Player;

fn spawn_player(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>
){
	
	commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(-5.0, 5.0, -5.0)).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
	commands.spawn_bundle(LightBundle {
            transform: Transform::from_xyz(-1.0, 0.0, 5.0),
            ..Default::default()
    });
	//only need this here:
	let player = commands.spawn()
		.insert(Player)
		.insert_bundle(PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
			material: materials.add(Color::WHITE.into()),
			transform: Transform::from_translation(Vec3::ZERO),
			..Default::default()
		}).id();
	println!("{}", player.id());
}

fn control_player(
	mut commands: Commands,
	mut ev_cursor_motion: EventReader<CursorMoved>,
	mut query: Query<(&Player, &mut Transform)>
){
	if let Some(cursor) = ev_cursor_motion.iter().last() {
		let (player, mut transform) = query.single_mut().expect("There is always a player");
		transform.rotate(Quat::from_rotation_y(cursor.position.x));
	}
}

fn main() {
    App::build()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(Msaa { samples: 4 })
		
        .add_plugins(DefaultPlugins)
        
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        
        .add_plugin(PhysicsPlugin::default())
		.insert_resource(Gravity::from(Vec3::new(0.0, -9.81, 0.0)))
        
        .add_plugin(DefaultRaycastingPlugin::<MyRaycastSet>::default())
        
        //Setup world
		.add_startup_system(spawn_player.system())
		
		//main play loop
		.add_system(control_player.system())
		
        .run();
}