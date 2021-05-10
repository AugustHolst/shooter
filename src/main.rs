use bevy::{
	pbr::AmbientLight, prelude::*,
	input::{keyboard::*, mouse::*, Input, ElementState},
	render::camera::*,
	diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}
};
use bevy_mod_raycast::{DefaultRaycastingPlugin, RayCastMesh, RayCastMethod, RayCastSource, RaycastSystem, Ray3d};
use heron::prelude::*;


// Entity tags
struct Player;
struct Shooter;

// RayCast Sets
struct ScreenSpaceSet;
struct ShootableSet;

// Resources
struct CursorInWorld(Vec3);// : Cursor ray intersection point

////
//	Startup systems -			start
fn startup_camera (mut commands: Commands) {
	//spawn camera
	commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(-5.0, 5.0, -5.0)).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
		})
		.insert(RayCastSource::<ScreenSpaceSet>::new());
} 

fn startup_world(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>
){
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
		.insert(RayCastMesh::<ScreenSpaceSet>::default())
		.insert(RayCastMesh::<ShootableSet>::default());
	
	//spawn big box
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(shape::Cube { size: 10.0 })),
			material: materials.add(Color::PINK.into()),
			transform: Transform::from_matrix(Mat4::from_rotation_translation(Quat::from_rotation_x(std::f32::consts::PI/2.0), Vec3::new(0.0, 0.0, 7.5))),
			..Default::default()
		})
		.insert(Body::Cuboid { half_extends: Vec3::new(5.0, 5.0, 5.0) })
		.insert(BodyType::Static)
		.insert(RayCastMesh::<ScreenSpaceSet>::default())
		.insert(RayCastMesh::<ShootableSet>::default());
}

fn startup_player(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>
){
	//Entity for shooting: consisting empty transform and RayCastSource
	commands.spawn()
		.insert(Shooter)
		.insert(GlobalTransform::identity())
		.insert(Transform::from_xyz(0.0, 0.5, 0.0)) //Use a static const for initial position
		.insert(RayCastSource::<ShootableSet>::new_transform_empty());
	
	//Player Entity
	let player = commands.spawn()
		.insert(Player)
		.insert_bundle(PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
			material: materials.add(Color::TEAL.into()),
			transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)), //Use a static const for initial position
			..Default::default()
		})
		.insert(Body::Cuboid { half_extends: Vec3::ONE * 0.5})
		.insert(BodyType::Dynamic)
		.id();
		//println!("Player ID: {}", player.id());

}
//	Startup systems - 		end
////

////
//	Systems -					start
fn world_cursor(
	mut commands: Commands,
	mut ev_cursor_moved: EventReader<CursorMoved>,
	mut query_ray: Query<&mut RayCastSource<ScreenSpaceSet>>
) {
	let mut screenspace_src = match query_ray.single_mut() {
		Ok(src) 	=> src,
		Err(e) 		=> panic!("Screenspace RayCastSource missing")
	};
	if let Some(cursor_latest) = ev_cursor_moved.iter().last() {
        screenspace_src.cast_method = RayCastMethod::Screenspace(cursor_latest.position);
		if let Some(intersection) = screenspace_src.intersect_top() {
			let intersection_point = intersection.1.position();
			commands.insert_resource(CursorInWorld(intersection_point));
		}
	}
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
	cursor_in_world: Res<CursorInWorld>,
	
	mut q_set: QuerySet<(
		Query<(Entity, &mut Transform, With<Player>)>,
		Query<(&mut Transform, &mut RayCastSource<ShootableSet>, With<Shooter>)>
	)>
){
	let player_pos : Vec3;
	if let Ok((player_Ent, mut player_transform, _)) = q_set.q0_mut().single_mut() { 
		let mut move_vec = Vec3::ZERO;
		let rot_matrix = Mat3::from_rotation_y(std::f32::consts::PI/4.0);
		let speed = 2.35;
		
		if keyboard_input.pressed(KeyCode::W) { move_vec += rot_matrix * Vec3::Z * speed};
		if keyboard_input.pressed(KeyCode::A) { move_vec += rot_matrix * Vec3::X * speed};
		if keyboard_input.pressed(KeyCode::S) { move_vec -= rot_matrix * Vec3::Z * speed};
		if keyboard_input.pressed(KeyCode::D) { move_vec -= rot_matrix * Vec3::X * speed};
		
		let mut look_vec = cursor_in_world.0;
		look_vec.y = player_transform.translation.y;
		player_transform.look_at(look_vec, Vec3::Y);
		
		commands.entity(player_Ent).insert(Velocity::from_linear(move_vec));
		
		player_pos = player_transform.translation.clone();
	} else {
		panic!("Player missing!");
	} if let Ok((mut shooter_transform, mut shooter_src, _)) = q_set.q1_mut().single_mut() {  
		shooter_transform.translation = player_pos;
		shooter_transform.look_at(cursor_in_world.0, Vec3::Y);
		if mouse_btn.pressed(MouseButton::Left) {
			if let Some(shot_intersection) = shooter_src.intersect_top() {
				println!("shoot2\n Screenspace: {}, Player: {}", cursor_in_world.0, shot_intersection.1.position());
			} else {
				let shoot_ray = shooter_src.ray().expect("Ray is shooting");
				println!("Ray position and direction: {}, {}", shoot_ray.origin(), shoot_ray.direction());
			}
		}
	} else {
		panic!("Shooting source is missing!");
	}
}
//	Systems -					end
////


fn main() {
    App::build()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(Msaa { samples: 4 })
		
        .add_plugins(DefaultPlugins)
        
        .add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        
        .add_plugin(PhysicsPlugin::default())
		.insert_resource(Gravity::from(Vec3::new(0.0, -9.81, 0.0)))
		
        .add_plugin(DefaultRaycastingPlugin::<ScreenSpaceSet>::default())
		.add_plugin(DefaultRaycastingPlugin::<ShootableSet>::default())
		
		.insert_resource(CursorInWorld(Vec3::ONE))
		
        // Startup systems
		.add_startup_system(startup_camera.system())
		.add_startup_system(startup_world.system())
		.add_startup_system(startup_player.system())
		
		// Unlabeled systems
		.add_system(bevy::input::system::exit_on_esc_system.system())
		
		// Labeled systems
		.add_system(world_cursor.system().label("world_cursor"))
		.add_system(control_player.system().label("control_player").after("world_cursor"))
		
        .run();
}