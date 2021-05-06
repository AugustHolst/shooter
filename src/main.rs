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
		.insert(Body::Cuboid { half_extends: Vec3::new(7.5, 0.0, 7.5) })
		.insert(BodyType::Static)
		.insert(RayCastMesh::<MyRaycastSet>::default())
		.insert(RayCastMesh::<Player>::default());
	
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 10.0 })),
			material: materials.add(Color::PINK.into()),
			transform: Transform::from_matrix(Mat4::from_rotation_translation(Quat::from_rotation_x(std::f32::consts::PI/2.0), Vec3::new(0.0, 0.0, 7.5))),
			..Default::default()
		})
		.insert(Body::Cuboid { half_extends: Vec3::new(5.0, 5.0, 5.0) })
		.insert(BodyType::Static)
		.insert(RayCastMesh::<MyRaycastSet>::default())
		.insert(RayCastMesh::<Player>::default());
		
	//only need this here:
	let player = commands.spawn()
		.insert(Player)
		.insert_bundle(PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
			material: materials.add(Color::TEAL.into()),
			transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
			..Default::default()
		})
		.insert(Body::Cuboid { half_extends: Vec3::ONE * 0.5})
		.insert(BodyType::Dynamic)
		.insert(RayCastSource::<Player>::new_transform_empty())
		.id();
	println!("{}", player.id());
}

/* Player Control System:
	Uses WASD to move the player.
	Rotates the player on the Y-axis using a ray shot from the cursor's position.
*/
fn control_player(
	time: Res<Time>,
	keyboard_input: Res<Input<KeyCode>>,
	mouse_btn: Res<Input<MouseButton>>,
	mut commands: Commands,
	
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	
	mut ev_cursor_motion: EventReader<CursorMoved>,
	mut query: Query<(Entity, &mut Transform, &mut RayCastSource<Player>, With<Player>)>,
	mut query_ray: Query<&mut RayCastSource<MyRaycastSet>>
){
	if let Ok((player_Ent, mut transform, mut shoot_src, _)) = query.single_mut() {
		let mut intersection_pos = Vec3::ZERO; 
		let mut look_vec = Vec3::ZERO; 
		
		let mut move_vec = Vec3::ZERO;
		let rot_matrix = Mat3::from_rotation_y(std::f32::consts::PI/4.0);
		
		if keyboard_input.pressed(KeyCode::W) { move_vec += rot_matrix * Vec3::Z };
		if keyboard_input.pressed(KeyCode::A) { move_vec += rot_matrix * Vec3::X };
		if keyboard_input.pressed(KeyCode::S) { move_vec -= rot_matrix * Vec3::Z };
		if keyboard_input.pressed(KeyCode::D) { move_vec -= rot_matrix * Vec3::X };
		
		commands.entity(player_Ent).insert(Velocity::from_linear(move_vec));
		
		
		if let Some(cursor) = ev_cursor_motion.iter().last() {
			for mut ray_src in &mut query_ray.iter_mut() {
				ray_src.cast_method = RayCastMethod::Screenspace(cursor.position);
				if let Some(intersection) = ray_src.intersect_top() {
					intersection_pos = intersection.1.position();
					look_vec = intersection_pos.clone();
					look_vec.y = transform.translation.y;
					transform.look_at(look_vec, Vec3::Y);
				}
				
				if mouse_btn.pressed(MouseButton::Left) {
					println!("shoot1");
					println!("{}", intersection_pos);
					let shoot_dir = (intersection_pos - transform.translation).normalize();
					let shoot_ray = Ray3d::new(transform.translation, shoot_dir).to_transform();
					let shoot_src = shoot_src.with_ray_transform(shoot_ray);
					if let Some(shot_intersection) = shoot_src.intersect_top() {
						println!("shoot2\n Screenspace: {}, Player: {}", intersection_pos, shot_intersection.1.position());
					} else {
						let shoot_ray = shoot_src.ray().expect("Ray is shooting");
						println!("Ray position and direction: {}, {}", shoot_ray.origin(), shoot_ray.direction());
					}
				}
			}
		}
	} else {
		println!("Player probably missing, so it crashed...");
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
		.add_plugin(DefaultRaycastingPlugin::<Player>::default())
        
        //Setup world
		.add_startup_system(spawn_player.system())
		
		//main play loop
		.add_system(bevy::input::system::exit_on_esc_system.system())
		.add_system(control_player.system())
		
        .run();
}