use bevy::prelude::*;
use pvw_rrect_physics::*;

const TILE_SIZE: f32 = 40.0;
const TILE_SIZE_VEC: Vec2 = Vec2::splat(TILE_SIZE);

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(PvwRRectPhysicsPlugin::default());
    app.insert_resource(TileSize::new(TILE_SIZE));
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            player_movement,
            message.run_if(on_message::<CollisionMessage>),
        ),
    );
    app.run()
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct HeavyBox;

#[derive(Component)]
struct LightBox;

#[derive(Component)]
struct Wall;

fn setup(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2d));

    commands.spawn((
        Name::new("Player"),
        Player,
        Position::default(),
        Collider::new(Vec2::ONE, 0.2, ColliderType::Dynamic(1.0)),
        Sprite::from_color(Color::srgb(0.0, 0.0, 1.0), TILE_SIZE_VEC),
    ));

    commands.spawn((
        Name::new("Heavy Box"),
        HeavyBox,
        Position(vec2(5.0, 0.0)),
        Collider::new(Vec2::splat(2.0), 0.4, ColliderType::Dynamic(4.0)), // Larger mass, pushed slower
        Sprite::from_color(Color::srgb(0.6, 0.4, 0.0), TILE_SIZE_VEC * 2.0),
    ));

    commands.spawn((
        Name::new("Light Box"),
        LightBox,
        Position(vec2(-5.0, 0.0)),
        Collider::new(Vec2::ONE, 0.2, ColliderType::Dynamic(0.5)), // Smaller mass, pushed faster
        Sprite::from_color(Color::srgb(0.5, 0.4, 0.0), TILE_SIZE_VEC),
    ));

    commands.spawn((
        Name::new("Wall"),
        Wall,
        Position(vec2(0.0, 5.0)),
        Collider::new(vec2(5.0, 1.0), 0.0, ColliderType::Static), // Static, cannot be pushed
        Sprite::from_color(Color::srgb(0.3, 0.3, 0.3), vec2(5.0 * TILE_SIZE, TILE_SIZE)),
    ));
}

fn player_movement(
    mut query: Query<&mut Movement, With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut player) = query.single_mut() else {
        return;
    };

    let mut force = Vec2::ZERO;

    if input.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        force.y += 1.0;
    }

    if input.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        force.x -= 1.0;
    }

    if input.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        force.y -= 1.0;
    }

    if input.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        force.x += 1.0;
    }

    force = force.normalize_or_zero() * 5.0;

    player.apply_force(PartialForce {
        id: "player_movement".to_string(),
        active: Some(true),
        force: Some(force),
    });
}

fn message(mut msgs: MessageReader<CollisionMessage>) {
    for m in msgs.read() {
        println!("{:?}", m);
    }
}
