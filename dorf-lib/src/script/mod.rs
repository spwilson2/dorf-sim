use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use crate::{
    prelude::*,
    terminal::{
        camera::{CameraResized, TerminalCamera2d},
        render::CharTexture,
        render::Transform,
    },
};

use bevy::input::ButtonState;
use bevy::{input::keyboard::KeyboardInput, transform};
use ordered_float::OrderedFloat;

#[derive(Default)]
pub struct ScriptPlugin();

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CollisionGridCache::new(UVec2 { x: 640, y: 640 }))
            .add_system(handle_camera_movement_keys)
            .add_system(handle_camera_resized)
            .add_system(system_assign_optimal_path)
            .add_system(system_move_on_optimal_path)
            .add_system(sys_update_collision_cache)
            .add_system(sys_handle_collisions)
            .add_startup_system(spawn_collider_walls)
            .add_startup_system(spawn_mv_player)
            .add_startup_system(spawn_textures);
    }
}

#[derive(Component)]
struct MovePath {
    steps: Vec<Vec2>,
}

#[derive(Component)]
struct GoalLoc(Vec2);
#[derive(Component)]
struct Speed(f32);

#[derive(Debug)]
struct Grid2D<T> {
    data: Vec<T>,
    size: UVec2,
}

impl<T: Clone> Grid2D<T> {
    fn new(size: UVec2, fill: T) -> Self {
        Self {
            data: vec![fill; (size.x * size.y) as usize],
            size,
        }
    }
}
impl<T> Grid2D<T> {
    /// Panics if point out of range
    #[inline]
    fn get(&self, point: UVec2) -> &T {
        let idx = self.idx_for_point(point);
        &self.data[idx]
    }
    #[inline]
    fn idx_for_point(&self, point: UVec2) -> usize {
        (point.y * self.size.x + point.y) as usize
    }
    /// Panics if point out of range
    #[inline]
    fn set_idx(&mut self, idx: usize, entity: T) {
        self.data[idx] = entity;
    }
    /// Panics if point out of range
    #[inline]
    fn set(&mut self, point: UVec2, entity: T) {
        let idx = self.idx_for_point(point);
        self.data[idx] = entity;
    }
}

//#[derive(Debug, Default, PartialEq)]
//struct UVec2 {
//    x: u32,
//    y: u32,
//}

#[derive(Resource, Debug)]
struct CollisionGridCache {
    grid: Grid2D<Option<Entity>>,
    entities: HashMap<Entity, Transform>,
}

fn for_points_on_transform<F>(transform: &Transform, mut f: F)
where
    F: FnMut(UVec2),
{
    let rect = Rect::from_center_size(transform.loc, transform.scale);
    for x in (rect.min.x as u32)..(rect.max.x as u32) {
        for y in (rect.min.y as u32)..(rect.max.y as u32) {
            f(UVec2 { x, y })
        }
    }
}

impl CollisionGridCache {
    fn new(size: UVec2) -> Self {
        Self {
            grid: Grid2D::new(size, None),
            entities: default(),
        }
    }
    #[inline]
    fn move_entity(&mut self, transform: &Transform, entity: Entity) {
        // Note: Works on the assumption that there may only be a single
        // collidable on a given point.
        if let Some(old_transform) = self.entities.insert(entity, transform.clone()) {
            for_points_on_transform(&old_transform, |point| self.grid.set(point, None))
        }
        for_points_on_transform(transform, move |point| self.grid.set(point, Some(entity)))
    }
    #[inline]
    fn clear(&mut self) {}
}

//impl From<Vec2> for UVec2 {
//    fn from(vec2: Vec2) -> Self {
//        Self {
//            x: vec2.x as u32,
//            y: vec2.y as u32,
//        }
//    }
//}

/// Simple tag Component indicating an entity is collidable
#[derive(Component, Debug, Default)]
struct BoxCollider {}

fn sys_update_collision_cache(
    mut cache: ResMut<CollisionGridCache>,
    q: Query<(Entity, &Transform, &BoxCollider), Changed<Transform>>,
) {
    for (entity, transform, _collider) in q.iter() {
        // TODO z_level
        cache.move_entity(transform, entity);
    }
}

fn sys_handle_collisions(
    cache: Res<CollisionGridCache>,
    q: Query<(Entity, &Transform, &BoxCollider), Changed<Transform>>,
) {
    for (entity, transform, _collider) in q.iter() {
        // TODO z_level
        if let Some(existing_entity) = cache.grid.get(UVec2 {
            x: transform.loc.x as u32,
            y: transform.loc.y as u32,
        }) {
            if *existing_entity != entity {
                log::error!("Panic! Overlapping entities!");
                panic!("Overlapping entities");
            }
        }
    }
}

#[derive(Bundle)]
struct Player {
    speed: Speed,
    goal: GoalLoc,
    rect: CharTexture,
    transform: Transform,
    collider: BoxCollider,
}

#[derive(Debug)]
struct AStar2DSearchState {
    calculated: HashMap<IVec2, OrderedFloat<f32>>,
    to_explore: BinaryHeap<Reverse<FunctionalTuple>>,
}

#[derive(PartialEq, Debug)]
struct FunctionalTuple(OrderedFloat<f32>, IVec2);

impl Eq for FunctionalTuple {}
impl Ord for FunctionalTuple {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl PartialOrd for FunctionalTuple {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl AStar2DSearchState {
    fn new(start_node: IVec2) -> Self {
        let mut s = Self {
            calculated: default(),
            to_explore: default(),
        };
        s.calculated.insert(start_node, 0.0.into());
        s
    }
    #[inline]
    fn calc_heuristic_of_point(node: Vec2, goal: &Vec2) -> f32 {
        goal.distance(node)
    }
    #[inline]
    fn explore_point(&mut self, point: IVec2, goal: &Vec2, cost: f32) {
        if !self.calculated.contains_key(&point) {
            let functional =
                AStar2DSearchState::calc_heuristic_of_point(point.as_vec2(), goal) + cost;
            self.calculated.insert(point, cost.into());
            self.to_explore
                .push(Reverse(FunctionalTuple(functional.into(), point)));
        }
    }
    fn explore_neighbors(&mut self, node: IVec2, goal: &Vec2, cost: f32) {
        let left = IVec2 { x: -1, y: 0 } + node;
        let right = IVec2 { x: 1, y: 0 } + node;
        let up = IVec2 { x: 0, y: 1 } + node;
        let down = IVec2 { x: 0, y: -1 } + node;
        let upleft = IVec2 { x: -1, y: 1 } + node;
        let upright = IVec2 { x: 1, y: 1 } + node;
        let downleft = IVec2 { x: -1, y: -1 } + node;
        let downright = IVec2 { x: 1, y: -1 } + node;
        // Note: Could speed up even more by only exploring in direction?
        self.explore_point(left, goal, 1.0 + cost);
        self.explore_point(right, goal, 1.0 + cost);
        self.explore_point(up, goal, 1.0 + cost);
        self.explore_point(down, goal, 1.0 + cost);
        //self.explore_point(upleft, goal, 1.0 + cost);
        //self.explore_point(upright, goal, 1.0 + cost);
        //self.explore_point(downleft, goal, 1.0 + cost);
        //self.explore_point(downright, goal, 1.0 + cost);
    }
    fn cheapest_neighbor(&mut self, node: IVec2) -> IVec2 {
        let left = IVec2 { x: -1, y: 0 } + node;
        let right = IVec2 { x: 1, y: 0 } + node;
        let up = IVec2 { x: 0, y: 1 } + node;
        let down = IVec2 { x: 0, y: -1 } + node;

        let mut neighbors = [
            (FunctionalTuple(*self.calculated.get(&up).unwrap_or(&f32::MAX.into()), up)),
            (FunctionalTuple(
                *self.calculated.get(&down).unwrap_or(&f32::MAX.into()),
                down,
            )),
            (FunctionalTuple(
                *self.calculated.get(&right).unwrap_or(&f32::MAX.into()),
                right,
            )),
            (FunctionalTuple(
                *self.calculated.get(&left).unwrap_or(&f32::MAX.into()),
                left,
            )),
        ];
        neighbors.sort();
        neighbors.first().unwrap().1
    }

    fn select_next_node(&mut self) -> Option<(f32, IVec2)> {
        self.to_explore
            .pop()
            .and_then(|h| Some((self.calculated.get(&h.0 .1).unwrap().0, h.0 .1)))
    }
}
fn calc_optimal_path(col_cache: &CollisionGridCache, start: Vec2, goal: Vec2) -> Option<Vec<Vec2>> {
    // TODO: A*

    // TODO: Start at first node, find next best node towards goal
    let mut state = AStar2DSearchState::new(start.as_ivec2());
    let mut cur_node = start.as_ivec2();
    let mut cost = 0.0;
    loop {
        // Collect costs for all neighbors
        state.explore_neighbors(cur_node, &goal, cost);
        // Select the next best node to explore, save the actual value
        (cost, cur_node) = state.select_next_node()?;
        if cur_node == goal.as_ivec2() {
            break;
        }
    }
    let mut path = Vec::new();
    let mut next = goal.as_ivec2();
    loop {
        next = state.cheapest_neighbor(next);
        path.push(next.as_vec2());
        if next == start.as_ivec2() {
            break;
        }
    }
    Some(path)
}

fn system_assign_optimal_path(
    mut cmd: Commands,
    col_cache: Res<CollisionGridCache>,
    q: Query<(Entity, &Transform, &GoalLoc)>,
) {
    for (entity, transform, goal) in q.iter() {
        // TODO: Calculate optimal path, for now will work since nothing to
        // colldie with. Will need to figure out how to do efficient collision
        // detec later.
        let path = calc_optimal_path(&*col_cache, transform.loc, goal.0);
        cmd.entity(entity)
            .insert(MovePath {
                steps: path.unwrap(), // TODO: Handle inability to path.
            })
            .remove::<GoalLoc>();
    }
}

fn system_move_on_optimal_path(
    mut cmd: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut MovePath, &mut Transform, &Speed)>,
) {
    log::info!("in move optimal");
    for (entity, mut path, mut rect, speed) in q.iter_mut() {
        let mut travel = speed.0 * time.delta().as_secs_f32();
        loop {
            // While we have time to travel, contiue doing so.

            // First check if there's nothing left to move
            if path.steps.is_empty() {
                cmd.entity(entity).remove::<MovePath>();
                // TODO: Add another goal path
                break;
            }

            let dist = rect.loc.distance(*path.steps.last().unwrap());
            if dist <= travel {
                travel -= dist;
                // We've move to the point, need to continue to the next point.
                rect.loc = path.steps.pop().unwrap();
            } else {
                // We can't move directly to the point, let's get as close as we can
                let direction = (*path.steps.last().unwrap() - rect.loc).normalize();
                rect.loc += direction * travel;
                break;
            }
        }
    }
}

fn spawn_mv_player(mut cmd: Commands) {
    cmd.spawn(Player {
        speed: Speed(1.5),
        goal: GoalLoc(Vec2::new(10.0, 10.0)),
        rect: CharTexture { texture: 'p' },
        transform: Transform {
            scale: Vec2::new(1.0, 1.0),
            loc: Vec2::new(0.0, 0.0),
            z_lvl: 2,
        },
        collider: default(),
    });
}

#[derive(Bundle)]
struct ColliderWall {
    texture: CharTexture,
    transform: Transform,
    collider: BoxCollider,
}

fn spawn_collider_walls(mut cmd: Commands) {
    cmd.spawn(ColliderWall {
        texture: CharTexture { texture: '-' },
        transform: Transform {
            scale: Vec2 { x: 1.0, y: 4.0 },
            loc: Vec2 { x: 4.0, y: 4.0 },
            z_lvl: 2,
        },
        collider: default(),
    });
}
/// - Create a player with a random intended path
///    - new entity:
///         - new CurLocation
///         - new GoalLocation
/// - Get the actual steps to take to get to that path using a*
///    - ref CurLocation
///    - del GoalLocation
///    - new TargetLocation
/// - Move the player to that position along the routed path
///    - mut CurLocation
///    -
/// - Player at destitation, then give them a new random path

fn spawn_textures(mut cmd: Commands) {
    let vert_wall = CharTexture { texture: '-' };
    let vert_wall_trans = Transform {
        scale: Vec2::new(1.0, 2.0),
        loc: Vec2::new(0.0, 0.0),
        z_lvl: 1000,
    };
    let side_wall = CharTexture { texture: '|' };
    let side_wall_trans = Transform {
        scale: Vec2::new(2.0, 1.0),
        loc: Vec2::new(0.0, 0.0),
        z_lvl: 1000,
    };
    cmd.spawn((
        CharTexture { texture: 'a' },
        Transform {
            scale: Vec2::new(1.0, 1.0),
            loc: Vec2::new(0.0, 0.0),
            z_lvl: 1,
        },
    ));
    cmd.spawn_batch([
        CameraFrameWallBundle {
            texture: side_wall.clone(),
            transform: side_wall_trans.clone(),
            side: CameraSide::Right,
        },
        CameraFrameWallBundle {
            texture: side_wall,
            transform: side_wall_trans.clone(),
            side: CameraSide::Left,
        },
        CameraFrameWallBundle {
            texture: vert_wall.clone(),
            transform: vert_wall_trans,
            side: CameraSide::Top,
        },
        CameraFrameWallBundle {
            texture: vert_wall,
            transform: side_wall_trans,
            side: CameraSide::Bottom,
        },
    ]);
}

/// Marker component type to indicate the CameraFrame Entity.
#[derive(Component, Default)]
struct CameraFrame {}

#[derive(Bundle)]
struct CameraFrameWallBundle {
    texture: CharTexture,
    transform: Transform,
    side: CameraSide,
}

#[derive(Component)]
enum CameraSide {
    Left,
    Right,
    Top,
    Bottom,
}

fn handle_camera_resized(
    mut walls: Query<(&mut Transform, &CameraSide)>,
    camera: Res<TerminalCamera2d>,
    mut event: EventReader<CameraResized>,
) {
    if let Some(event) = event.iter().last() {
        center_camera_frame(&camera, &mut walls)
    }
}

fn center_camera_frame(
    camera: &TerminalCamera2d,
    walls: &mut Query<(&mut Transform, &CameraSide)>,
) {
    for mut wall in walls.iter_mut() {
        match *wall.1 {
            CameraSide::Left => {
                wall.0.loc.x = -camera.dim().x / 2.0 + 1.0 + camera.loc().x;
                wall.0.loc.y = camera.loc().y;
                wall.0.scale.x = 1.0;
                wall.0.scale.y = camera.dim().y;
            }
            CameraSide::Right => {
                wall.0.loc.x = camera.dim().x / 2.0 + camera.loc().x;
                wall.0.loc.y = camera.loc().y;
                wall.0.scale.x = 1.0;
                wall.0.scale.y = camera.dim().y;
            }
            CameraSide::Top => {
                wall.0.loc.x = camera.loc().x;
                wall.0.loc.y = -camera.dim().y / 2.0 + 1.0 + camera.loc().y;
                wall.0.scale.x = camera.dim().x;
                wall.0.scale.y = 1.0;
            }
            CameraSide::Bottom => {
                wall.0.loc.x = camera.loc().x;
                wall.0.loc.y = camera.dim().y / 2.0 + camera.loc().y;
                wall.0.scale.x = camera.dim().x;
                wall.0.scale.y = 1.0;
            }
        }
    }
}

fn move_camera(
    direction: Vec2,
    camera: &mut ResMut<TerminalCamera2d>,
    walls: &mut Query<(&mut Transform, &CameraSide)>,
) {
    camera.move_by(Vec3::new(direction.x, direction.y, 0.0));
    center_camera_frame(&*camera, walls);
}

fn handle_camera_movement_keys(
    mut input: EventReader<KeyboardInput>,
    mut camera: ResMut<TerminalCamera2d>,
    mut walls: Query<(&mut Transform, &CameraSide)>,
) {
    for e in input.iter() {
        if e.state != ButtonState::Pressed {
            continue;
        }
        if let Some(k) = e.key_code {
            match k {
                KeyCode::D => move_camera(Vec2::new(1.0, 0.0), &mut camera, &mut walls),
                KeyCode::A => move_camera(Vec2::new(-1.0, 0.0), &mut camera, &mut walls),
                KeyCode::W => move_camera(Vec2::new(0.0, -1.0), &mut camera, &mut walls),
                KeyCode::S => move_camera(Vec2::new(0.0, 1.0), &mut camera, &mut walls),
                _ => (),
            }
        }
    }
}
