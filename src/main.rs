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
		.insert(RayCastMesh::<MyRaycastSet>::default());
	
	
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
		.id();
	println!("{}", player.id());
}

fn control_player(
	mut commands: Commands,
	mut ev_cursor_motion: EventReader<CursorMoved>,
	mut ev_keyboard: EventReader<KeyboardInput>,
	mut query: Query<(Entity, &mut Transform, With<Player>)>,
	mut query_ray: Query<&mut RayCastSource<MyRaycastSet>>
){
	let (player_Ent, mut transform, _) = query.single_mut().expect("There is always a player");
	let internsection_pos : Vec3;
	let (mut w_pressed, mut a_pressed, mut s_pressed, mut d_pressed) = (false, false, false, false);
	for kb_input in ev_keyboard.iter() {
		match kb_input.key_code.unwrap() {
			KeyCode::W =>	{	w_pressed = true},
								
			KeyCode::A => 	{	a_pressed = true},
								
			KeyCode::S => 	{	s_pressed = true},
								
			KeyCode::D => 	{	d_pressed = true},
			_ 			=> {println!("?")}
		}
	}
	let mut move_vec = Vec3::ZERO;
	let rot_matrix = Mat3::from_rotation_y(std::f32::consts::PI/4.0);
	if w_pressed || a_pressed || s_pressed || d_pressed {
		if w_pressed { move_vec += rot_matrix * Vec3::Z };
		if a_pressed { move_vec += rot_matrix * Vec3::X };
		if s_pressed { move_vec -= rot_matrix * Vec3::Z };
		if d_pressed { move_vec -= rot_matrix * Vec3::X };
		println!("{}", move_vec.to_string());
	}
	
	commands.entity(player_Ent).insert(Velocity::from_linear(move_vec));
	
	
	if let Some(cursor) = ev_cursor_motion.iter().last() {
		for mut ray_src in &mut query_ray.iter_mut() {
			ray_src.cast_method = RayCastMethod::Screenspace(cursor.position);
			if let Some(intersection) = ray_src.intersect_top() {
				let mut intersection_pos = intersection.1.position();
				intersection_pos.y = transform.translation.y;
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