use bevy::{
	pbr::AmbientLight, prelude::*,
	input::{keyboard::*, mouse::*, Input, ElementState},
	render::camera::*,
};

fn main() {
    App::build()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(Msaa { samples: 4 })
		
        .add_plugins(DefaultPlugins)
		
        .add_startup_system(setup.system())
		.add_startup_system(hello_world.system())
		.add_startup_system(spawn_camera.system())
		
		.add_system(pan_orbit_camera.system())
		.add_system(spawn_cube.system())
        .run();
}



/// set up a simple 3D scene
fn setup (
	mut commands: Commands, 
	mut meshes: ResMut<Assets<Mesh>>, 
	mut materials: ResMut<Assets<StandardMaterial>>
){
	commands.
		spawn_bundle(PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
			material: materials.add(Color::WHITE.into()),
			..Default::default()
		});
	
    commands
        .spawn_bundle(LightBundle {
            transform: Transform::from_xyz(-1.0, 0.0, 5.0),
            ..Default::default()
        });
}

/// Tags an entity as capable of panning and orbiting.
struct PanOrbitCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

fn hello_world() {
	println!("Hello");
} 

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
fn pan_orbit_camera(
    windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
	mut ev_keyboard: EventReader<KeyboardInput>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &PerspectiveProjection)>,
) {
    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Middle;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.iter() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.iter() {
            pan += ev.delta;
        }
    }
    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }
	
    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
		for kb_event in ev_keyboard.iter() {
			if kb_event.state == ElementState::Pressed {
				let rot_matrix = Mat3::from_quat(transform.rotation);
				match kb_event.key_code.unwrap() {
					KeyCode::W => {	transform.translation += rot_matrix * Vec3::unit_y();
										pan_orbit.focus 		+= rot_matrix * Vec3::unit_y()},
										
					KeyCode::A => {	transform.translation -= rot_matrix * Vec3::unit_x();
										pan_orbit.focus 		-= rot_matrix * Vec3::unit_x()},
										
					KeyCode::S => {	transform.translation -= rot_matrix * Vec3::unit_y();
										pan_orbit.focus 		-= rot_matrix * Vec3::unit_y()},
										
					KeyCode::D => {	transform.translation += rot_matrix * Vec3::unit_x();
										pan_orbit.focus 		+= rot_matrix * Vec3::unit_x()},
					_ => println!("?")
				}
			}
		}
		
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let window = get_primary_window_size(&windows);
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down { -delta } else { delta }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            let window = get_primary_window_size(&windows);
            pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
			let translation_z_rad = Vec3::new(0., 0., pan_orbit.radius);
            transform.translation = pan_orbit.focus + rot_matrix.mul_vec3(translation_z_rad);
        }
    }
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}

/// Spawn a camera like this
fn spawn_camera(mut commands: Commands) {
    let translation = Vec3::new(-2.0, 2.5, 5.0);
    let radius = translation.length();

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(translation)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    }).insert(PanOrbitCamera {
        radius,
        ..Default::default()
    });
}

fn spawn_cube(
	mut commands: Commands,
	mut ev_motion: EventReader<MouseMotion>,
	input_mouse: Res<Input<MouseButton>>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>
){
	let left_mouse = MouseButton::Left;
	if input_mouse.just_pressed(left_mouse) {
		commands.
			spawn_bundle(PbrBundle {
				mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0})),
				transform: Transform::from_translation(Vec3::Y * 10.0),
				..Default::default()
			});
	}
}