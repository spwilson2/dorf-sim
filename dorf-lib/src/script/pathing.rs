use std::{
    cmp::{max, min, Reverse},
    collections::{BinaryHeap, HashMap},
};

use crate::{
    prelude::*,
    script::{local_map::LOCAL_MAP_DIMMENSIONS, pathing},
};

use bevy::{input::keyboard::KeyboardInput, transform};
use bevy::{input::ButtonState, utils::Uuid};
use ordered_float::OrderedFloat;

/// A component indicating a location to move towards. Once a `MovePath` has
/// been assigned, the contained `Vec2` point will be cleared and set to `None`.
#[derive(Component)]
pub struct GoalLoc(Option<Vec2>);

/// A component indicating an Entity's move speed.
#[derive(Component)]
pub struct Speed(f32);

/// A tag Component indicating an entity is collidable but will not move
#[derive(Component, Debug, Default)]
pub struct ImmobileObstacle;

/// A tag Component indicating an entity is moveable, should be verified it
/// doesn't land on an `ImmobileObstacleTag`, and but it can layer with other
/// `LayerableCollider` entities
#[derive(Component, Debug, Default)]
pub struct LayerableCollider;

/// A compnent to containing a computed path to a previously assigned `GoalLoc`
#[derive(Component)]
struct MovePath {
    steps: Vec<Vec2>,
}

/// A cache used to store the location of static Entities which objects should avoid.
#[derive(Resource, Debug)]
struct CollisionGridCache {
    grid: Grid2D<Option<Entity>>,
    entities: HashMap<Entity, Transform2D>,
}

/// Initialization function for pathing systems and an example spawner.
pub fn add_pathing_systems(app: &mut App, enabled: bool) {
    if enabled {
        log::debug!("pathing system: enabled");
        app.insert_resource(CollisionGridCache::new(
            IVec2::default(),
            LOCAL_MAP_DIMMENSIONS,
        ))
        .add_system(pathing::system_move_on_optimal_path)
        .add_system(pathing::sys_update_collision_cache)
        .add_system(pathing::system_assign_optimal_path.after(pathing::sys_update_collision_cache))
        .add_system(pathing::sys_handle_collisions)
        .add_system(pathing::spawn_mv_player_over_time)
        .add_startup_system(pathing::spawn_collider_walls);
    } else {
        log::debug!("pathing system: disabled");
    }
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
    pub fn new(center: IVec2, size: UVec2) -> Self {
        Self {
            grid: Grid2D::new(center, size, None),
            entities: default(),
        }
    }
    fn dbg_dump_to_log(&self) {
        let mut string = String::new();

        let min = self.grid.rect().min;
        let max = self.grid.rect().max;

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
                        match self.grid.get(IVec2 { x, y }).unwrap() {
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

    /// Returns:
    /// - `Ok(true)` if `point` would collide with cached colliders, `Ok(false)` if not
    /// - `Err(OutOfBoundsError)` if the point isn't on the grid
    #[inline]
    pub fn collides(&self, point: IVec2) -> Result<bool, crate::LightError> {
        Ok(self.grid.get(point)?.is_some())
    }
    #[inline]
    pub fn transform_collides_with(
        &self,
        obj: &Transform2D,
        uuid: Entity,
    ) -> Result<Option<Entity>, crate::LightError> {
        // Check overlapping rect
        let mut col = None;
        for_points_on_transform(obj, |point| {
            let res = self.grid.get(point)?;
            if res.is_some() {
                if *res != Some(uuid) {
                    col = *res;
                    return Err(LightError::TerminateEarly);
                }
            }
            Ok(())
        });
        Ok(col)
    }

    /// Returns:
    /// - `Ok(true)` if `obj` would collide with cached colliders if it were
    ///    moved to `point`, `Ok(false)` if not.
    /// - `Err(OutOfBoundsError)` if the point isn't on the grid
    pub fn would_collide_if_moved(
        &self,
        obj: &Transform2D,
        new_loc: &IVec2,
    ) -> Result<bool, LightError> {
        let mut obj = obj.clone();
        obj.loc = new_loc.xyy().as_vec3();
        // Check overlapping rect
        let res = for_points_on_transform(&obj, |point| {
            if self.collides(point)? {
                return Err(LightError::TerminateEarly);
            }
            Ok(())
        });
        match res {
            Err(LightError::TerminateEarly) => return Ok(true),
            Ok(_) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Updates the cache by moving the specific entity's colliders
    #[inline]
    fn move_entity(&mut self, transform: &Transform2D, uuid: Entity) {
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
            if self.grid.get(point)?.is_none() {
                self.grid.set(point, Some(uuid));
            }
            Ok::<(), LightError>(())
        });
    }
}

fn sys_update_collision_cache(
    mut cache: ResMut<CollisionGridCache>,
    q: Query<(Entity, &Transform2D, &ImmobileObstacle), Changed<Transform2D>>,
) {
    for (entity, transform, _obstacle) in q.iter() {
        log::info!("Moving {:?}", entity);
        // TODO z_level
        cache.move_entity(transform, entity);
    }
}

fn sys_handle_collisions(
    cache: Res<CollisionGridCache>,
    q: Query<(Entity, &Transform2D, &LayerableCollider), Changed<Transform2D>>,
) {
    for (entity, transform, collider) in q.iter() {
        if let Some(uuid) = cache.transform_collides_with(transform, entity).unwrap() {
            log::error!("Panic! Entity touching wall! {:?}", transform);
            panic!("Overlapping entities");
        }
    }
}

#[derive(Debug)]
struct AStar2DSearchState<'i> {
    calculated: HashMap<IVec2, OrderedFloat<f32>>,
    calculated_: Grid2D<Option<OrderedFloat<f32>>>,
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
            calculated_: Grid2D::new(
                col_cache.grid.rect().min,
                col_cache.grid.rect().size().as_uvec2(),
                None,
            ),
            to_explore: default(),
            col_cache,
        };
        //s.calculated.insert(start_node.as_tile(), 0.0.into());
        s.calculated_.set(start_node.as_tile(), Some(0.0.into()));
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
    }

    #[inline]
    fn calc_heuristic_of_point(node: Vec2, goal: &Vec2) -> f32 {
        goal.distance(node)
    }
    #[inline]
    fn explore_point(&mut self, start: &Transform2D, new_point: IVec2, goal: &Vec2, mut cost: f32) {
        //if !self.calculated.contains_key(&new_point) {
        if let Ok(known) = self.calculated_.get(new_point) {
            if known.is_some() {
                return;
            }
            if self
                .col_cache
                .would_collide_if_moved(start, &new_point)
                // If fails to unwrap, then it's because we looked at an out-of-bounds point.
                .unwrap_or_else(|e| {
                    debug_assert_eq!(e, LightError::OutOfBoundsError);
                    true
                })
            {
                cost = f32::NAN;
            } else {
                let functional =
                    AStar2DSearchState::calc_heuristic_of_point(new_point.as_vec2(), goal) + cost;
                self.to_explore.push(Reverse(FunctionalTuple(
                    OrderedFloat(functional),
                    new_point,
                )));
            }
            self.calculated_.set(new_point, Some(OrderedFloat(cost)));
            //self.calculated.insert(new_point, OrderedFloat(cost));
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
            (FunctionalTuple(
                self.calculated_
                    .get(up)
                    .unwrap_or(&Some(f32::MAX.into()))
                    .unwrap_or(f32::MAX.into()),
                up,
            )),
            (FunctionalTuple(
                self.calculated_
                    .get(down)
                    .unwrap_or(&Some(f32::MAX.into()))
                    .unwrap_or(f32::MAX.into()),
                down,
            )),
            (FunctionalTuple(
                self.calculated_
                    .get(right)
                    .unwrap_or(&Some(f32::MAX.into()))
                    .unwrap_or(f32::MAX.into()),
                right,
            )),
            (FunctionalTuple(
                self.calculated_
                    .get(left)
                    .unwrap_or(&Some(f32::MAX.into()))
                    .unwrap_or(f32::MAX.into()),
                left,
            )),
        ];
        neighbors.sort();
        neighbors.first().unwrap().1
    }

    fn select_next_node(&mut self) -> Option<(f32, IVec2)> {
        self.to_explore
            .pop()
            //.and_then(|h| Some((self.calculated.get(&h.0 .1).unwrap().0, h.0 .1)))
            .and_then(|h| Some((self.calculated_.get(h.0 .1).unwrap().unwrap().0, h.0 .1)))
    }
}

/// Calculate the optimal path from `start` to `goal` using A* search algorithm
fn calc_optimal_path(
    col_cache: &CollisionGridCache,
    start: &Transform2D,
    goal: Vec2,
) -> Option<Vec<Vec2>> {
    let mut state = AStar2DSearchState::new(col_cache, start);
    let mut cur_node = start.clone();
    let mut cost = 0.0;
    let mut next_loc;
    loop {
        // Collect costs for all neighbors
        state.explore_neighbors(&cur_node, &goal, cost);
        // Select the next best node to explore, carry the cost of getting to
        // that node. If there aren't any more nodes to explore then there was
        // no path so return None.
        (cost, next_loc) = state.select_next_node()?;
        cur_node.loc = next_loc.as_vec2().xyy();
        // If the next node gets us to the goal, we're done
        if cur_node.as_tile() == goal.as_ivec2() {
            break;
        }
    }

    // Now just collect the cheapest path from the goal to where we started
    let mut path = Vec::new();
    let mut next = goal.as_ivec2();
    loop {
        next = state.cheapest_neighbor(next);
        path.push(next.as_vec2());
        if next == start.as_tile() {
            break;
        }
    }
    Some(path)
}

fn random_point_on_local_map() -> Vec2 {
    Vec2::new(fastrand::f32(), fastrand::f32()) * LOCAL_MAP_DIMMENSIONS.as_vec2()
}

/// System which will act on Entities wth `Some(GoalLoc)` and compute an optimal
/// `MovePath` using A* search.
fn system_assign_optimal_path(
    mut cmd: Commands,
    col_cache: Res<CollisionGridCache>,
    mut q: Query<(Entity, &Transform2D, &mut GoalLoc), Changed<GoalLoc>>,
) {
    for (entity, transform, mut goal) in q.iter_mut() {
        if let Some(goal_loc) = goal.0 {
            match calc_optimal_path(&*col_cache, transform, goal_loc) {
                // Path Found
                Some(path) => {
                    cmd.entity(entity).insert(MovePath { steps: path });
                    goal.0 = None;
                }
                // No path found
                None => {
                    goal.0 = Some(random_point_on_local_map());
                }
            }
        }
    }
}

/// System that will move Entities along their given `MovePath`, once they reach
/// the end of their assignments, then assign a new goal.
fn system_move_on_optimal_path(
    time: Res<Time>,
    mut q: Query<(
        Entity,
        &mut MovePath,
        &mut Transform2D,
        &Speed,
        &mut GoalLoc,
    )>,
) {
    for (entity, mut path, mut rect, speed, mut goal) in q.iter_mut() {
        let mut travel = speed.0 * time.delta().as_secs_f32();
        // While we have time to travel, contiue doing so.
        loop {
            // First check if there's nothing left to move, in which case we're
            // done. Just assign a new goal.
            if path.steps.is_empty() {
                goal.0 = Some(random_point_on_local_map());
                break;
            }
            let dist = rect.loc.xy().distance(*path.steps.last().unwrap());
            // Check if we can simply move to the point, with our currently alloted travel distance.
            if dist <= travel {
                travel -= dist;
                let res = path.steps.pop().unwrap();
                (rect.loc.x, rect.loc.y) = (res[0], res[1]);
                // We've moved to the point, need to loop back around to the next point.
                continue;
            }
            // We can't move directly to the point, let's get as close as we can.
            let direction = (*path.steps.last().unwrap() - rect.loc.xy()).normalize();
            rect.loc.x += direction.x * travel;
            rect.loc.y += direction.y * travel;
            break;
        }
    }
}

#[derive(Bundle)]
struct Player {
    speed: Speed,
    goal: GoalLoc,
    rect: CharTexture,
    transform: Transform2D,
    collider: LayerableCollider,
}

#[derive(Bundle)]
struct ColliderWall {
    texture: CharTexture,
    transform: Transform2D,
    collider: ImmobileObstacle,
}

fn spawn_mv_player_over_time(mut cmd: Commands, mut cnt: Local<usize>) {
    if *cnt > 1000 {
        return;
    }
    *cnt += 1;
    cmd.spawn(Player {
        speed: Speed(5.0),
        goal: GoalLoc(Some(Vec2::new(0.0, 1.0))),
        rect: CharTexture::new('▢', Color::RED),
        transform: Transform2D {
            scale: UVec2::splat(1),
            loc: Vec3::ZERO,
        },
        collider: default(),
    });
}

fn spawn_collider_walls(mut cmd: Commands) {
    cmd.spawn(ColliderWall {
        texture: CharTexture::from_char('▢'),
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
