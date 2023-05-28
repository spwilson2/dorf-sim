use std::{
    cmp::{max, min, Reverse},
    collections::{BinaryHeap, HashMap},
};

use crate::{
    prelude::*,
    terminal::{
        camera::{CameraResized, TerminalCamera2d},
        render::CharTexture,
    },
};

use bevy::{
    input::keyboard::KeyboardInput,
    math::{Vec2Swizzles, Vec3Swizzles},
    transform,
};
use bevy::{input::ButtonState, utils::Uuid};
use ordered_float::OrderedFloat;

#[derive(Component)]
pub struct MovePath {
    steps: Vec<Vec2>,
}

#[derive(Component)]
pub struct GoalLoc(Vec2);
#[derive(Component)]
pub struct Speed(f32);

#[derive(Debug)]
pub struct Grid2D<T> {
    data: Vec<T>,
    rect: Rect,
}

impl<T: Clone> Grid2D<T> {
    fn new(center: IVec2, size: IVec2, fill: T) -> Self {
        Self {
            data: vec![fill; (size.x * size.y) as usize],
            rect: Rect::from_center_size(center.as_vec2(), size.as_vec2()),
        }
    }
}
impl<T> Grid2D<T> {
    /// Panics if point out of range
    #[inline]
    fn get(&self, point: IVec2) -> &T {
        let idx = self.idx_for_point(point);
        &self.data[idx]
    }
    #[inline]
    fn idx_for_point(&self, point: IVec2) -> usize {
        // TODO: Recenter

        let bot_left = self.rect.min.as_ivec2();
        // Distance_y * size_x  + Distance_x
        ((point.y - bot_left.y) * self.rect.size().as_ivec2().x + (point.x - bot_left.x)) as usize
    }
    /// Panics if point out of range
    #[inline]
    fn set_idx(&mut self, idx: usize, entity: T) {
        self.data[idx] = entity;
    }
    /// Panics if point out of range
    #[inline]
    fn set(&mut self, point: IVec2, entity: T) {
        let idx = self.idx_for_point(point);
        self.data[idx] = entity;
    }
}

//#[derive(Debug, Default, PartialEq)]
//struct UVec2 {
//    x: u32,
//    y: u32,
//}

struct StopIteration;

#[derive(Resource, Debug)]
pub struct CollisionGridCache {
    grid: Grid2D<Option<Uuid>>,
    entities: HashMap<Uuid, Transform2D>,
}

fn for_points_on_transform<F, E>(transform: &Transform2D, f: F) -> Result<(), E>
where
    F: FnMut(IVec2) -> Result<(), E>,
{
    for_points_in_rect(&Rect2D::from_transform2d(transform), f)
}

fn for_points_in_rect<F, E>(rect: &Rect2D, mut f: F) -> Result<(), E>
where
    F: FnMut(IVec2) -> Result<(), E>,
{
    for x in (rect.min.x as i32)..(rect.max.x as i32) {
        for y in (rect.min.y as i32)..(rect.max.y as i32) {
            f(IVec2 { x: x, y: y })?
        }
    }
    Ok(())
}

impl CollisionGridCache {
    #[inline]
    pub fn new(center: IVec2, size: IVec2) -> Self {
        Self {
            grid: Grid2D::new(center, size, None),
            entities: default(),
        }
    }
    fn dbg_dump_to_log(&self) {
        let mut string = String::new();

        let min = self.grid.rect.min.as_ivec2();
        let max = self.grid.rect.max.as_ivec2();

        string.push_str("\n    ");
        for x in min.x..max.x {
            string.push_str(format!(" {}", x.abs() % 10).as_str());
        }
        for y in min.y..max.y {
            string.push_str(format!("\n{:04}", y).as_str());
            for x in min.x..max.x {
                string.push_str(
                    format!(
                        " {}",
                        match self.grid.get(IVec2 { x, y }) {
                            Some(_) => 'x',
                            None => '*',
                        }
                    )
                    .as_str(),
                );
            }
        }
        log::info!("Collision Grid: {}", string);
    }

    #[inline]
    pub fn collides(&self, point: IVec2) -> bool {
        self.grid.get(point).is_some()
    }
    #[inline]
    pub fn transform_collides_with(&self, obj: &Transform2D, uuid: Uuid) -> Option<Uuid> {
        // Check overlapping rect
        let mut col = None;
        for_points_on_transform(obj, |point| {
            let res = self.grid.get(point);
            if res.is_some() {
                if *res != Some(uuid) {
                    col = *res;
                    return Err(());
                }
            }
            Ok(())
        });
        col
    }
    pub fn would_collide_if_moved(&self, obj: &Transform2D, new_loc: &IVec2) -> bool {
        let mut obj = obj.clone();
        obj.loc = new_loc.xyy().as_vec3();

        // Check overlapping rect
        for_points_on_transform(&obj, |point| {
            if self.collides(point) {
                Err(())
            } else {
                Ok(())
            }
        })
        .is_err()
    }
    #[inline]
    fn move_entity(&mut self, transform: &Transform2D, uuid: Uuid) {
        // Note: Works on the assumption that there may only be a single
        // collidable on a given point.
        if let Some(old_transform) = self.entities.insert(uuid, transform.clone()) {
            for_points_on_transform(&old_transform, |point| {
                self.grid.set(point, None);
                Ok::<(), ()>(())
            });
        }
        // Check if another exists, if so don't override. Collision detection will report.
        for_points_on_transform(transform, move |point| {
            if self.grid.get(point).is_none() {
                self.grid.set(point, Some(uuid));
            }
            Ok::<(), ()>(())
        });
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
#[derive(Component, Debug)]
pub struct BoxCollider {
    uuid: Uuid,
}

impl Default for BoxCollider {
    fn default() -> Self {
        Self {
            uuid: bevy::utils::Uuid::new_v4(),
        }
    }
}

pub fn sys_update_collision_cache(
    mut cache: ResMut<CollisionGridCache>,
    q: Query<(&Transform2D, &BoxCollider), Changed<Transform2D>>,
) {
    for (transform, collider) in q.iter() {
        log::info!("Moving {:?}", collider.uuid);
        // TODO z_level
        cache.move_entity(transform, collider.uuid);
    }
}

pub fn sys_handle_collisions(
    cache: Res<CollisionGridCache>,
    q: Query<(&Transform2D, &BoxCollider), Changed<Transform2D>>,
) {
    for (transform, collider) in q.iter() {
        if let Some(uuid) = cache.transform_collides_with(transform, collider.uuid) {
            // TODO z_level
            //if let Some(existing_uuid) = cache.grid.get(IVec2 {
            //    x: transform.loc.x as i32,
            //    y: transform.loc.y as i32,
            //}) {
            log::error!("Panic! Overlapping entities! {:?}", transform);
            panic!("Overlapping entities");
        }
    }
}

#[derive(Bundle)]
struct Player {
    speed: Speed,
    goal: GoalLoc,
    rect: CharTexture,
    transform: Transform2D,
    collider: BoxCollider,
}

#[derive(Debug)]
struct AStar2DSearchState<'i> {
    calculated: HashMap<IVec2, OrderedFloat<f32>>,
    to_explore: BinaryHeap<Reverse<FunctionalTuple>>,
    col_cache: &'i CollisionGridCache,
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

impl<'i> AStar2DSearchState<'i> {
    fn new(col_cache: &'i CollisionGridCache, start_node: &Transform2D) -> Self {
        let mut s = Self {
            calculated: default(),
            to_explore: default(),
            col_cache,
        };
        s.calculated.insert(start_node.as_tile(), 0.0.into());
        s
    }
    fn dbg_dump_to_log(&self) {
        // iterate through all to explore and all calculated to get bounds.
        // Then print

        let mut iter = self.calculated.keys();
        let point = iter.next().unwrap();
        let (mut max_x, mut max_y) = (point.x, point.y);
        let (mut min_x, mut min_y) = (point.x, point.y);
        for point in iter {
            min_x = min(min_x, point.x);
            min_y = min(min_y, point.y);
            max_x = max(max_x, point.x);
            max_y = max(max_y, point.y);
        }

        let mut string = String::new();
        string.push_str("\n    ");
        for x in min_x..max_x {
            string.push_str(format!(" {}", x.abs() % 10).as_str());
        }
        for y in min_y..max_y {
            string.push_str(format!("\n{:04}", y).as_str());
            for x in min_x..max_x {
                string.push_str(
                    format!(
                        " {}",
                        match self.calculated.get(&IVec2 { x, y }) {
                            Some(val) => {
                                if val.0 == f32::MAX {
                                    'x'
                                } else {
                                    'o'
                                }
                            }
                            None => '*',
                        }
                    )
                    .as_str(),
                );
            }
        }
        log::info!("Search Grid: {}", string);
        //calculated: HashMap<IVec2, OrderedFloat<f32>>,
        //to_explore: BinaryHeap<Reverse<FunctionalTuple>>,
        //col_cache: &'i CollisionGridCache,
    }

    #[inline]
    fn calc_heuristic_of_point(node: Vec2, goal: &Vec2) -> f32 {
        goal.distance(node)
    }
    #[inline]
    fn explore_point(&mut self, start: &Transform2D, new_point: IVec2, goal: &Vec2, cost: f32) {
        if !self.calculated.contains_key(&new_point) {
            let functional;
            if self.col_cache.would_collide_if_moved(start, &new_point) {
                functional = f32::MAX;
            } else {
                functional =
                    AStar2DSearchState::calc_heuristic_of_point(new_point.as_vec2(), goal) + cost;
            }
            self.calculated.insert(new_point, cost.into());
            self.to_explore
                .push(Reverse(FunctionalTuple(functional.into(), new_point)));
        }
    }
    fn explore_neighbors(&mut self, transform: &Transform2D, goal: &Vec2, cost: f32) {
        let node = transform.as_tile();
        let left = IVec2 { x: -1, y: 0 } + node;
        let right = IVec2 { x: 1, y: 0 } + node;
        let up = IVec2 { x: 0, y: 1 } + node;
        let down = IVec2 { x: 0, y: -1 } + node;
        let upleft = IVec2 { x: -1, y: 1 } + node;
        let upright = IVec2 { x: 1, y: 1 } + node;
        let downleft = IVec2 { x: -1, y: -1 } + node;
        let downright = IVec2 { x: 1, y: -1 } + node;
        // Note: Could speed up even more by only exploring in direction?
        self.explore_point(&transform, left, goal, 1.0 + cost);
        self.explore_point(&transform, right, goal, 1.0 + cost);
        self.explore_point(&transform, up, goal, 1.0 + cost);
        self.explore_point(&transform, down, goal, 1.0 + cost);
        self.explore_point(&transform, upleft, goal, 1.4 + cost);
        self.explore_point(&transform, upright, goal, 1.4 + cost);
        self.explore_point(&transform, downleft, goal, 1.4 + cost);
        self.explore_point(&transform, downright, goal, 1.4 + cost);
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
fn calc_optimal_path(
    col_cache: &CollisionGridCache,
    start: &Transform2D,
    goal: Vec2,
) -> Option<Vec<Vec2>> {
    // TODO: A*

    // TODO: Start at first node, find next best node towards goal
    let mut state = AStar2DSearchState::new(col_cache, start);
    let mut cur_node = start.clone();
    let mut cost = 0.0;
    let mut next_loc;
    loop {
        // Collect costs for all neighbors
        state.explore_neighbors(&cur_node, &goal, cost);
        // Select the next best node to explore, save the actual value
        (cost, next_loc) = state.select_next_node()?;
        cur_node.loc = next_loc.as_vec2().xyy();
        if cur_node.as_tile() == goal.as_ivec2() {
            break;
        }
    }
    let mut path = Vec::new();
    let mut next = goal.as_ivec2();
    loop {
        next = state.cheapest_neighbor(next);
        path.push(next.as_vec2());
        if next == start.as_tile() {
            break;
        }
    }
    state.dbg_dump_to_log();
    Some(path)
}

pub fn system_assign_optimal_path(
    mut cmd: Commands,
    col_cache: Res<CollisionGridCache>,
    q: Query<(Entity, &Transform2D, &GoalLoc)>,
) {
    if !q.is_empty() {
        col_cache.dbg_dump_to_log();
    }
    for (entity, transform, goal) in q.iter() {
        // TODO: Calculate optimal path, for now will work since nothing to
        // colldie with. Will need to figure out how to do efficient collision
        // detec later.
        let path = calc_optimal_path(&*col_cache, transform, goal.0);
        cmd.entity(entity)
            .insert(MovePath {
                steps: path.unwrap(), // TODO: Handle inability to path.
            })
            .remove::<GoalLoc>();
    }
}

pub fn system_move_on_optimal_path(
    mut cmd: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut MovePath, &mut Transform2D, &Speed)>,
) {
    // TODO: Need to handle recompute if colliders change
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

            let dist = rect.loc.xy().distance(*path.steps.last().unwrap());
            if dist <= travel {
                travel -= dist;
                // We've moved to the point, need to continue to the next point.
                let res = path.steps.pop().unwrap();
                (rect.loc.x, rect.loc.y) = (res[0], res[1]);
                //rect.loc.xy() = ;
            } else {
                // We can't move directly to the point, let's get as close as we can
                let direction = (*path.steps.last().unwrap() - rect.loc.xy()).normalize();
                rect.loc.x += direction.x * travel;
                rect.loc.y += direction.y * travel;
                break;
            }
        }
    }
}

pub(crate) fn spawn_mv_player(mut cmd: Commands) {
    cmd.spawn(Player {
        speed: Speed(1.5),
        goal: GoalLoc(Vec2::new(10.0, 10.0)),
        rect: CharTexture { texture: 'p' },
        transform: Transform2D {
            scale: UVec2::splat(1),
            loc: Vec3::ZERO,
        },
        collider: default(),
    });
}

#[derive(Bundle)]
struct ColliderWall {
    texture: CharTexture,
    transform: Transform2D,
    collider: BoxCollider,
}

pub(crate) fn spawn_collider_walls(mut cmd: Commands) {
    cmd.spawn(ColliderWall {
        texture: CharTexture { texture: 'x' },
        transform: Transform2D {
            scale: UVec2 { x: 1, y: 5 },
            loc: Vec3 {
                x: 3.0,
                y: 0.0,
                z: 0.0,
            },
        },
        collider: default(),
    });
}
