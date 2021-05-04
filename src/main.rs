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
	//spawn camera
	commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(-5.0, 5.0, -5.0)).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
		})
		.insert(RayCastSource::<MyRaycastSet>::new());
		
	//spawn light
	commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(-1.0, 2.0, 5.0),
        ..Default::default()
		});
	
	//spawn planes
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Plane { size: 15.0 })),
			material: materials.add(Color::WHITE.into()),
			..Default::default()
		})
		.insert(Body::Cuboid { half_extends: Vec3::new(2.5, 0.0, 2.5) })
		.insert(BodyType::Static)
		.insert(RayCastMesh::<MyRaycastSet>::default());
	
	
	//only need this here:
	let player = commands.spawn()
		.insert(Player)
		.insert_bundle(PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
			material: materials.add(Color::TEAL.into()),
			transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
			..Default::default()
		}).id();
	println!("{}", player.id());
}

fn control_player(
	mut commands: Commands,
	mut ev_cursor_motion: EventReader<CursorMoved>,
	mut query: Query<(&Player, &mut Transform)>,
	mut query_ray: Query<&mut RayCastSource<MyRaycastSet>>
){
	if let Some(cursor) = ev_cursor_motion.iter().last() {
		let (player, mut transform) = query.single_mut().expect("There is always a player");
		for mut ray_src in &mut query_ray.iter_mut() {
			ray_src.cast_method = RayCastMethod::Screenspace(cursor.position);
			if let Some(intersection) = ray_src.intersect_top() {
				let intersection_pos = intersection.1.position();
				transform.look_at(intersection_pos, Vec3::Y);
			}
		}
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