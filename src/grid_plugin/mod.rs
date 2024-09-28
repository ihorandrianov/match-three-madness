use bevy::prelude::*;
use rand::{rngs::ThreadRng, Rng};

use crate::view_plugin::Selected;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OneSecondTimer>();
        app.add_systems(
            Startup,
            (setup_grid, check_for_matches, reroll_matched).chain(),
        );
        app.add_systems(
            Update,
            (
                swap_two_selected,
                check_for_matches,
                reroll_matched.run_if(check_if_settled),
                remove_matched.run_if(oposite_if_settled),
                drop_down,
                spawn_new,
            )
                .chain(),
        );
    }
}

#[derive(Component)]
pub struct Tile;

#[derive(Component, Debug)]
pub struct Position(pub Vec2);

impl Position {
    fn new(x: usize, y: usize) -> Self {
        let position = Vec2::new(x as f32, y as f32);
        Self(position)
    }
}

#[derive(Component, Eq, PartialEq, Hash)]
pub enum TileType {
    Red,
    Green,
    Pink,
    Tea,
    Cookie,
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Match;

#[derive(Resource)]
pub struct GridSize {
    rows: usize,
    cols: usize,
}

impl GridSize {
    pub fn get_size(&self) -> Vec2 {
        Vec2::new(self.cols as f32, self.rows as f32)
    }
}

#[derive(Resource, PartialEq, Eq)]
enum StartGridState {
    DIRTY,
    SETTLED,
}

#[derive(Resource)]
pub struct GridMap(pub Vec<Vec<Option<Entity>>>);

pub fn setup_grid(mut commands: Commands) {
    let rows = 8;
    let cols = 8;
    let mut rng = rand::thread_rng();
    let mut grid: Vec<Vec<Option<Entity>>> = vec![vec![None; rows]; cols];
    commands.insert_resource(GridSize { rows, cols });
    for y in 0..rows {
        for x in 0..cols {
            let tile_type = make_random_tiletype(&mut rng);
            let position = Position::new(x, y);
            let entity = commands.spawn((tile_type, position, Tile));
            grid[y][x] = Some(entity.id());
        }
    }
    println!("{:?}", grid);
    commands.insert_resource(GridMap(grid));
    commands.insert_resource(StartGridState::DIRTY);
}

pub fn check_for_matches(
    grid_size: Res<GridSize>,
    mut commands: Commands,
    query: Query<(Entity, &Position, &TileType), With<Tile>>,
) {
    let mut grid: Vec<Vec<Option<(Entity, &TileType)>>> =
        vec![vec![None; grid_size.cols]; grid_size.rows];

    for tile in query.iter() {
        let position = tile.1;
        grid[position.0.x as usize][position.0.y as usize] = Some((tile.0, tile.2));
    }
    // Check for matches row
    for y in 0..grid_size.rows {
        for x in 0..grid_size.cols {
            if let Some(current_tile) = grid[y][x] {
                let right_n = grid[y].get(x + 1).and_then(|x| x.as_ref());
                let right_n2 = grid[y].get(x + 2).and_then(|x| x.as_ref());

                let bottom_n = grid.get(y + 1).and_then(|y| y[x].as_ref());
                let bottom_n2 = grid.get(y + 2).and_then(|y| y[x].as_ref());
                // check right
                if let (Some(right_tile), Some(right_tile2)) = (right_n, right_n2) {
                    if right_tile.1 == current_tile.1 && right_tile2.1 == current_tile.1 {
                        commands.entity(right_tile.0).insert(Match);
                        commands.entity(current_tile.0).insert(Match);
                        commands.entity(right_tile2.0).insert(Match);
                    }
                }

                if let (Some(bottom_tile), Some(bottom_tile2)) = (bottom_n, bottom_n2) {
                    if bottom_tile.1 == current_tile.1 && bottom_tile2.1 == current_tile.1 {
                        commands.entity(current_tile.0).insert(Match);
                        commands.entity(bottom_tile.0).insert(Match);
                        commands.entity(bottom_tile2.0).insert(Match);
                    }
                }
            };
        }
    }
}

fn reroll_matched(
    tiles: Query<Entity, With<Match>>,
    mut commands: Commands,
    start_grid_state: ResMut<StartGridState>,
) {
    let mut rng = rand::thread_rng();
    if tiles.iter().count() == 0 {
        let start_grid_state = start_grid_state.into_inner();
        *start_grid_state = StartGridState::SETTLED;
        return;
    }
    for entity in tiles.iter() {
        let new_tile_type = make_random_tiletype(&mut rng);

        commands
            .entity(entity)
            .insert(new_tile_type)
            .remove::<Match>();
    }
}

fn make_random_tiletype(rng: &mut ThreadRng) -> TileType {
    match rng.gen_range(0..6) {
        0 => TileType::Red,
        1 => TileType::Green,
        2 => TileType::Pink,
        3 => TileType::Tea,
        _ => TileType::Cookie,
    }
}

fn check_if_settled(start_grid_state: Res<StartGridState>) -> bool {
    if start_grid_state.into_inner() == &StartGridState::DIRTY {
        return true;
    }
    return false;
}

fn oposite_if_settled(start_grid_state: Res<StartGridState>) -> bool {
    if start_grid_state.into_inner() == &StartGridState::DIRTY {
        return false;
    }
    return true;
}

fn swap_two_selected(
    mut tiles: Query<(Entity, &mut Position), (With<Tile>, With<Selected>)>,
    mut commands: Commands,
    mut grid: ResMut<GridMap>,
) {
    let mut combinator = tiles.iter_combinations_mut::<2usize>();
    if let Some([mut first, mut second]) = combinator.fetch_next() {
        let temp = [first.1 .0.x, first.1 .0.y];
        first.1 .0 = second.1 .0.clone();
        second.1 .0 = Vec2::new(temp[0], temp[1]);
        commands.entity(first.0).remove::<Selected>();
        commands.entity(second.0).remove::<Selected>();
        grid.0[second.1 .0.y as usize][second.1 .0.x as usize] = Some(second.0);
        grid.0[first.1 .0.y as usize][first.1 .0.x as usize] = Some(first.0);
        println!("swaped {:?}, {:?}", first.0, second.0);
    };
}

pub fn remove_matched(
    tiles: Query<(Entity, &Position), (With<Tile>, With<Match>)>,
    mut commands: Commands,
    grid: ResMut<GridMap>,
) {
    let grid = grid.into_inner();
    for (entity, position) in tiles.iter() {
        commands.entity(entity).despawn();
        grid.0[position.0.x as usize][position.0.y as usize] = None;
        println!(
            "{:?} Removed, {:?}",
            entity, grid.0[position.0.x as usize][position.0.y as usize]
        );
    }
}

fn drop_down(mut tiles: Query<(Entity, &mut Position), With<Tile>>, mut commands: Commands) {
    let mut grid: Vec<Vec<Option<Entity>>> = vec![vec![None; 8]; 8];
    for (entity, position) in tiles.iter() {
        grid[position.0.y as usize][position.0.x as usize] = Some(entity);
    }

    for y in 0..8 {
        for x in 0..8 {
            if y > 0 {
                let lower_row = grid.get(y - 1);
                let current_tile = grid[y][x];
                if let Some(lower_row) = lower_row {
                    let lower_tile = lower_row[x];
                    if let (None, Some(tile)) = (lower_tile, current_tile) {
                        grid[y - 1][x] = Some(tile);
                        grid[y][x] = None;
                        let position = tiles.get_mut(tile);
                        if let Ok((entity, _position)) = position {
                            commands.entity(entity).insert(Position::new(x, y - 1));
                        }
                    }
                }
            }
        }
    }
}

fn spawn_new(
    tiles: Query<&Position, With<Tile>>,
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<OneSecondTimer>,
) {
    if timer.0.tick(time.delta()).finished() {
        let mut top_row: Vec<Option<&Position>> = vec![None; 8];
        for tile in tiles.iter() {
            if tile.0.y == 7. {
                top_row[tile.0.x as usize] = Some(tile);
            }
        }

        let mut rng = rand::thread_rng();
        for (x, tile) in top_row.iter().enumerate() {
            if let None = tile {
                let tile_type = make_random_tiletype(&mut rng);
                commands.spawn((Position::new(x, 7), tile_type, Tile));
            }
        }
    }
}

#[derive(Resource)]
struct OneSecondTimer(Timer);

impl Default for OneSecondTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1., TimerMode::Repeating))
    }
}
