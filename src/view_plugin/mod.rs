use std::borrow::BorrowMut;

use bevy::{prelude::*, render::render_resource::Texture, utils::HashMap, window::PrimaryWindow};

use crate::grid_plugin::{Position, Tile, TileType};

pub struct ViewDPlugin;

impl Plugin for ViewDPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileProps>()
            .init_resource::<MyWorldCoords>()
            .insert_resource(ClearColor(Color::srgb(0.2, 0.3, 0.6)))
            .add_systems(Startup, (camera_setup, grid_sprite))
            .add_systems(
                Update,
                (draw_gems, my_cursor_system, collision_detect, set_selected).chain(),
            );
    }
}

#[derive(Component)]
struct GameCamera;

#[derive(Component)]
pub struct Selected;

fn camera_setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            ..Default::default()
        },
        GameCamera,
    ));
}

#[derive(Component)]
struct GridSprite;

fn grid_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tile_props: ResMut<TileProps>,
) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.4, 0.2, 0.4),
                custom_size: Some(Vec2::new(800., 800.)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0., 0., 0.),
                ..Default::default()
            },
            ..Default::default()
        },
        GridSprite,
    ));
    let red: Handle<Image> = asset_server.load("red.png");
    let green: Handle<Image> = asset_server.load("green.png");
    let pink: Handle<Image> = asset_server.load("pink.png");
    let tea: Handle<Image> = asset_server.load("tea.png");
    let cookie: Handle<Image> = asset_server.load("cookie.png");
    let props = tile_props.into_inner();
    props.0.insert(TileType::Red, red);
    props.0.insert(TileType::Tea, tea);
    props.0.insert(TileType::Cookie, cookie);
    props.0.insert(TileType::Pink, pink);
    props.0.insert(TileType::Green, green);
}

fn draw_gems(
    q_tiles: Query<(Entity, &Position, &TileType), With<Tile>>,
    mut commands: Commands,
    props: Res<TileProps>,
) {
    let tile_size = Vec2::new(80.0, 80.0);

    let grid_width = tile_size.x * 8.0;
    let grid_height = tile_size.y * 8.0;

    let offset_x = grid_width / 2.0;
    let offset_y = grid_height / 2.0;

    for (entity, position, tile_type) in q_tiles.iter() {
        let grid_position = position.0;
        let texture = props.0.get(tile_type).expect("texture exists").clone();
        let screen_x = (grid_position.x as f32 * tile_size.x) - offset_x;
        let screen_y = (grid_position.y as f32 * tile_size.y) - offset_y;
        let tile_size = tile_size - 10.;
        let sprite = match tile_type {
            &TileType::Red => SpriteBundle {
                texture,
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(tile_size),
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(screen_x, screen_y, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            &TileType::Green => SpriteBundle {
                texture,
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(tile_size),
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(screen_x, screen_y, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            &TileType::Pink => SpriteBundle {
                texture,
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(tile_size),
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(screen_x, screen_y, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            &TileType::Tea => SpriteBundle {
                texture,
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(tile_size),
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(screen_x, screen_y, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            _ => SpriteBundle {
                texture,
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(tile_size),
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(screen_x, screen_y, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        };

        commands.entity(entity).insert(sprite);
    }
}

#[derive(Resource, Default)]
struct MyWorldCoords(Vec2);

/// Used to help identify our main camera

fn my_cursor_system(
    mut mycoords: ResMut<MyWorldCoords>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mycoords.0 = world_position;
    }
}

fn collision_detect(
    coords: Res<MyWorldCoords>,
    mut q_tiles: Query<(&Transform, &mut Sprite), With<Tile>>,
) {
    for (transform, mut sprite) in q_tiles.iter_mut() {
        let position = transform.translation.truncate();
        let bounding_box = BoundingBox::from_center(position, 70.);
        if bounding_box.contains(&coords.0) {
            sprite.color.set_alpha(0.8);
        } else if !sprite.color.is_fully_opaque() {
            sprite.color.set_alpha(1.);
        }
    }
}

struct BoundingBox {
    min: Vec2,
    max: Vec2,
}

impl BoundingBox {
    fn from_center(center: Vec2, size: f32) -> Self {
        let half_size = size / 2.;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    fn contains(&self, point: &Vec2) -> bool {
        point.x > self.min.x && point.x < self.max.x && point.y > self.min.y && point.y < self.max.y
    }
}

fn set_selected(
    tiles: Query<(Entity, &Transform), With<Tile>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_coord: Res<MyWorldCoords>,
    mut commands: Commands,
) {
    if mouse.just_pressed(MouseButton::Left) {
        for (entity, transform) in tiles.iter() {
            let pos = transform.translation.truncate();
            let bb = BoundingBox::from_center(pos, 70.);
            if bb.contains(&mouse_coord.0) {
                commands.entity(entity).insert(Selected);
            }
        }
    }
}

#[derive(Resource)]
struct TileProps(HashMap<TileType, Handle<Image>>);

impl Default for TileProps {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
