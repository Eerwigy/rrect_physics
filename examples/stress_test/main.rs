use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::PrimaryWindow,
};
use pvw_rrect_physics::*;
use rand::Rng;

const TILE_SIZE: f32 = 40.0;
const TILE_SIZE_VEC: Vec2 = Vec2::splat(TILE_SIZE);

#[derive(Resource, Default)]
struct CursorPos {
    pub position: Vec2,
    cam_offset: Vec2,
}

#[derive(Message)]
struct SpawnBob(Vec2);

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    app.add_plugins(LogDiagnosticsPlugin::default());
    app.add_plugins(PvwRRectPhysicsPlugin {
        spatial_grid_size: 4.0, // Smaller grid size for more optimization
    });
    app.insert_resource(TileSize::new(TILE_SIZE));
    app.init_resource::<CursorPos>();
    app.add_message::<SpawnBob>();
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((Name::new("Camera"), Camera2d));
    });
    app.add_systems(
        Update,
        (
            update_cursor,
            should_bob_spawn.run_if(resource_changed::<ButtonInput<MouseButton>>),
            spawn_bob.run_if(on_message::<SpawnBob>),
            bob_collide.run_if(on_message::<CollisionMessage>),
        )
            .chain(),
    );
    app.run()
}

fn update_cursor(
    mut cursor: ResMut<CursorPos>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single().unwrap();
    let (camera, camera_transform) = camera.single().unwrap();
    let camera_pos = camera_transform.translation().xy();

    match window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        Some(cursor_pos) => {
            cursor.position = cursor_pos;
            cursor.cam_offset = cursor_pos - camera_pos;
        },

        None => {
            cursor.position = camera_pos + cursor.cam_offset;
        },
    }
}

fn should_bob_spawn(
    mut events: MessageWriter<SpawnBob>,
    click: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorPos>,
) {
    if click.just_pressed(MouseButton::Left) {
        events.write(SpawnBob(cursor.position / TILE_SIZE));
    }

    if click.just_pressed(MouseButton::Right) {
        for _ in 0..10 {
            events.write(SpawnBob(cursor.position / TILE_SIZE));
        }
    }
}

fn spawn_bob(mut commands: Commands, mut events: MessageReader<SpawnBob>) {
    let mut rng = rand::rng();

    for SpawnBob(pos) in events.read() {
        let mut movement = Movement::damped(Vec2::splat(0.8));
        movement.apply_force(PartialForce {
            id: "main".to_string(),
            force: Some(vec2(
                rng.random_range(-7.0..7.0), // Random velocity
                rng.random_range(-7.0..7.0),
            )),
            active: Some(false),
        });

        commands.spawn((
            Name::new("Bob"),
            Position(*pos),
            movement,
            Collider {
                ctype: ColliderType::Dynamic(rng.random_range(1.0..20.0)), // Random mass
                ..default()
            },
            Sprite::from_color(Color::srgb(1.0, 1.0, 0.0), TILE_SIZE_VEC),
        ));
    }
}

fn bob_collide(mut events: MessageReader<CollisionMessage>, query: Query<(), With<Collider>>) {
    println!(
        "{} collisions detected, bob count: {}",
        events.read().len(),
        query.iter().len()
    );
}
