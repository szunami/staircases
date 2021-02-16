use std::collections::{hash_map::Entry, HashMap, HashSet};

use bevy::{diagnostic::Diagnostics, prelude::*};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_resource(AdjacencyGraph::default())
        .add_startup_system(setup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        // .add_system(framerate.system())
        // reset IV
        .add_system(reset_intrinsic_velocity.system())
        // build edge graph
        .add_system(build_adjacency_graph.system())
        // assign step IV
        .add_system(step_intrinsic_velocity.system())
        // assign player IV
        .add_system(player_intrinsic_velocity.system())
        // assign falling IV
        .add_system(falling_intrinsic_velocity.system())
        // propagation
        .add_system(reset_velocity.system())
        .add_system(velocity_propagation.system())
        // reset velocity
        // for each IV, in order of ascending y, propagate
        .add_system(update_position.system())
        .add_system(update_step_arm.system())
        .run();
}

fn framerate(diagnostics: Res<Diagnostics>) {
    if let Some(fps) = diagnostics.get(bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
        dbg!(fps.average());
    }
}

#[derive(Clone)]
struct BoundingBox(Vec2);

struct Escalator;

struct Step {
    arm: Arm,
    escalator: Entity,
}

#[derive(Clone)]
enum Arm {
    A,
    B,
    C,
    D,
}

#[derive(Clone, PartialEq, Debug)]
struct Velocity(Option<Vec2>);

struct IntrinsicVelocity(Option<Propagation>);

#[derive(Clone, Debug)]
struct Propagation {
    push: Option<f32>,
    carry: Option<Vec2>,
    intrinsic: Option<Vec2>,
}

impl Default for Propagation {
    fn default() -> Self {
        Propagation {
            push: None,
            carry: None,
            intrinsic: None,
        }
    }
}

impl Propagation {
    fn to_velocity(&self) -> Vec2 {
        let push_x = self.push.unwrap_or_else(|| 0.0);
        let carry = self.carry.unwrap_or_else(Vec2::zero);
        let intrinsic = self.intrinsic.unwrap_or_else(Vec2::zero);

        let result_x = push_x + carry.x + intrinsic.x;
        let result_y = carry.y + intrinsic.y;

        Vec2::new(result_x, result_y)
    }
}

struct Ground;

#[derive(PartialEq, Eq, Hash)]
struct Crate;

struct Player;

struct AdjacencyGraph {
    lefts: HashMap<Entity, HashSet<Entity>>,
    rights: HashMap<Entity, HashSet<Entity>>,
    tops: HashMap<Entity, HashSet<Entity>>,
    bottoms: HashMap<Entity, HashSet<Entity>>,
}

impl Default for AdjacencyGraph {
    fn default() -> Self {
        AdjacencyGraph {
            lefts: HashMap::new(),
            rights: HashMap::new(),
            tops: HashMap::new(),
            bottoms: HashMap::new(),
        }
    }
}

fn setup(
    commands: &mut Commands,

    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,

    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default());

    let escalator_base = asset_server.load("textures/base.png");
    let escalator_atlas = TextureAtlas::from_grid(escalator_base, Vec2::new(200.0, 200.0), 1, 1);

    let escalator_handle = texture_atlases.add(escalator_atlas);
    let player_handle =
        materials.add(Color::rgb(115.0 / 255.0, 190.0 / 255.0, 211.0 / 255.0).into());
    let crate_handle = materials.add(Color::rgb(173.0 / 255.0, 119.0 / 255.0, 87.0 / 255.0).into());
    let ground_handle =
        materials.add(Color::rgb(87.0 / 255.0, 114.0 / 255.0, 119.0 / 255.0).into());
    let step_handle = materials.add(Color::rgb(168.0 / 255.0, 202.0 / 255.0, 88.0 / 255.0).into());

    {
        commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),

                        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
                        ..Default::default()
                    })
                    .with(Player {})
                    .with(BoundingBox(Vec2::new(50.0, 50.0)))
                    .with(Velocity(None))
                    .with(IntrinsicVelocity(None));

                let ground_box = Vec2::new(500.0, 50.0);
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                        sprite: Sprite::new(ground_box),
                        ..Default::default()
                    })
                    .with(Ground {})
                    .with(BoundingBox(ground_box))
                    .with(Velocity(None));

                let crate_box = Vec2::new(50.0, 50.0);

                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(-50.0, 50.0, 1.0)),
                        sprite: Sprite::new(crate_box),
                        ..Default::default()
                    })
                    .with(Crate {})
                    .with(BoundingBox(crate_box))
                    .with(IntrinsicVelocity(None))
                    .with(Velocity(None));

                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(-100.0, 50.0, 1.0)),
                        sprite: Sprite::new(crate_box),
                        ..Default::default()
                    })
                    .with(Crate {})
                    .with(BoundingBox(crate_box))
                    .with(IntrinsicVelocity(None))
                    .with(Velocity(None));
    }
}
fn spawn_escalator(
    commands: &mut Commands,
    texture: Handle<TextureAtlas>,
    transform: Transform,
    size: Vec2,
) -> Entity {
    commands
        .spawn(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                color: Color::rgba(1.0, 1.0, 1.0, 0.5),
                ..TextureAtlasSprite::default()
            },
            visible: Visible {
                is_visible: true,
                is_transparent: true,
            },
            texture_atlas: texture.clone_weak(),
            transform: transform,
            ..Default::default()
        })
        .with(Escalator {})
        .with(Velocity(None))
        .with(IntrinsicVelocity(None))
        .with(BoundingBox(size))
        .current_entity()
        .expect("escalator")
}

fn spawn_step(
    commands: &mut Commands,
    material: Handle<ColorMaterial>,
    escalator: Entity,
    transform: Transform,
    size: Vec2,
    arm: Arm,
) -> Entity {
    commands
        .spawn(SpriteBundle {
            material,
            transform,
            sprite: Sprite::new(size),
            ..Default::default()
        })
        .with(BoundingBox(size))
        .with(Step { arm, escalator })
        .with(Velocity(None))
        .with(IntrinsicVelocity(None))
        .current_entity()
        .expect("spawned step")
}

fn spawn_ground(
    commands: &mut Commands,
    material: Handle<ColorMaterial>,
    ground_box: Vec2,
    transform: Transform,
) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite::new(ground_box),
            material,
            transform,
            ..Default::default()
        })
        .with(BoundingBox(ground_box))
        .with(Ground);
}

fn spawn_player(
    commands: &mut Commands,
    material: Handle<ColorMaterial>,
    size: Vec2,
    transform: Transform,
) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite::new(size),
            transform,
            material,
            ..SpriteBundle::default()
        })
        .with(Player)
        .with(BoundingBox(size))
        .with(Velocity(None))
        .with(IntrinsicVelocity(None));
}

fn steps(escalator_transform: Transform, escalator_box: Vec2, step: Vec2) -> Vec<(Transform, Arm)> {
    let mut result = vec![];

    // A
    result.push((
        Transform::from_translation(Vec3::new(
            escalator_transform.translation.x - escalator_box.x / 2.0 + step.x / 2.0,
            escalator_transform.translation.y + escalator_box.y / 2.0 - step.y / 2.0,
            0.0,
        )),
        Arm::A,
    ));

    // B
    let n = (escalator_box.y / step.y) as i32;

    for index in 0..n - 2 {
        result.push((
            Transform::from_translation(Vec3::new(
                escalator_transform.translation.x - escalator_box.x / 2.0
                    + step.x / 2.0
                    + index as f32 * step.x,
                escalator_transform.translation.y + escalator_box.y / 2.0
                    - 3.0 * step.y / 2.0
                    - index as f32 * step.y,
                0.0,
            )),
            Arm::B,
        ))
    }

    // C
    result.push((
        Transform::from_translation(Vec3::new(
            escalator_transform.translation.x + escalator_box.x / 2.0 - 3.0 * step.y / 2.0,
            escalator_transform.translation.y - escalator_box.y / 2.0 + step.y / 2.0,
            0.0,
        )),
        Arm::C,
    ));

    // D
    for index in 0..n - 1 {
        result.push((
            Transform::from_translation(Vec3::new(
                escalator_transform.translation.x + escalator_box.x / 2.0
                    - step.x / 2.0
                    - (index as f32) * step.x,
                escalator_transform.translation.y
                    + -escalator_box.y / 2.0
                    + (step.y) / 2.0
                    + (index as f32) * step.y,
                0.0,
            )),
            Arm::D,
        ));
    }
    result
}

fn player_intrinsic_velocity(
    keyboard_input: Res<Input<KeyCode>>,
    adjacency_graph: Res<AdjacencyGraph>,
    mut query: Query<(&Player, Entity, &mut IntrinsicVelocity)>,
) {
    for (_player, entity, mut velocity) in query.iter_mut() {
        let mut x_velocity = 0.0;
        if keyboard_input.pressed(KeyCode::A) {
            x_velocity += -1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            x_velocity += 1.0;
        }

        let y_velocity = match adjacency_graph.bottoms.get(&entity) {
            Some(_) => 0.0,
            None => -1.0,
        };

        *velocity = IntrinsicVelocity(Some(Propagation {
            intrinsic: Some(Vec2::new(x_velocity, y_velocity)),
            ..Propagation::default()
        }));
    }
}

fn step_intrinsic_velocity(mut query: Query<(&Step, &mut IntrinsicVelocity)>) {
    for (step, mut intrinsic_velocity) in query.iter_mut() {
        match step.arm {
            Arm::A => {
                *intrinsic_velocity = IntrinsicVelocity(Some(Propagation {
                    intrinsic: Some(Vec2::new(0.0, -1.0)),
                    ..Propagation::default()
                }));
            }
            Arm::B => {
                *intrinsic_velocity = IntrinsicVelocity(Some(Propagation {
                    intrinsic: Some(Vec2::new(1.0, -1.0)),
                    ..Propagation::default()
                }));
            }
            Arm::C => {
                *intrinsic_velocity = IntrinsicVelocity(Some(Propagation {
                    intrinsic: Some(Vec2::new(1.0, 0.0)),
                    ..Propagation::default()
                }));
            }
            Arm::D => {
                *intrinsic_velocity = IntrinsicVelocity(Some(Propagation {
                    intrinsic: Some(Vec2::new(-1.0, 1.0)),
                    ..Propagation::default()
                }));
            }
        }
    }
}

fn update_position(mut query: Query<(&Velocity, &mut Transform)>) {
    for (maybe_velocity, mut transform) in query.iter_mut() {
        match maybe_velocity.0 {
            Some(velocity) => {
                transform.translation.x += velocity.x;
                transform.translation.y += velocity.y;
            }
            None => {
                // dbg!("Shouldn't happen in the future!");
            }
        }
    }
}

fn update_step_arm(
    mut steps: Query<(&mut Step, &BoundingBox, &Transform)>,
    escalators: Query<(&Escalator, &BoundingBox, &Transform)>,
) {
    for (mut step, step_box, step_transform) in steps.iter_mut() {
        let (_escalator, escalator_box, escalator_transform) =
            escalators.get(step.escalator).expect("fetch escalator");

        let step_top = step_transform.translation.y + step_box.0.y / 2.0;
        let step_bottom = step_transform.translation.y - step_box.0.y / 2.0;
        let step_right = step_transform.translation.x + step_box.0.x / 2.0;

        let escalator_top = escalator_transform.translation.y + escalator_box.0.y / 2.0;
        let escalator_bottom = escalator_transform.translation.y - escalator_box.0.y / 2.0;
        let escalator_right = escalator_transform.translation.x + escalator_box.0.x / 2.0;

        match step.arm {
            Arm::A => {
                if step_bottom < escalator_top - 2.0 * step_box.0.y {
                    step.arm = Arm::B;
                }
            }
            Arm::B => {
                if (step_bottom - escalator_bottom).abs() < std::f32::EPSILON {
                    step.arm = Arm::C;
                }
            }
            Arm::C => {
                if (step_right - escalator_right).abs() < std::f32::EPSILON {
                    step.arm = Arm::D;
                }
            }
            Arm::D => {
                if (step_top - escalator_top).abs() < std::f32::EPSILON {
                    step.arm = Arm::A;
                }
            }
        }
    }
}

fn is_atop(
    atop_transform: &Transform,
    atop_box: &BoundingBox,
    below_transform: &Transform,
    below_box: &BoundingBox,
) -> bool {
    let atop_bottom = atop_transform.translation.y - atop_box.0.y / 2.0;
    let atop_left = atop_transform.translation.x - atop_box.0.x / 2.0;
    let atop_right = atop_transform.translation.x + atop_box.0.x / 2.0;

    let below_top = below_transform.translation.y + below_box.0.y / 2.0;
    let below_left = below_transform.translation.x - below_box.0.x / 2.0;
    let below_right = below_transform.translation.x + below_box.0.x / 2.0;

    (atop_bottom - below_top).abs() < std::f32::EPSILON
        && ((atop_left <= below_left && below_left < atop_right)
            || (below_left <= atop_left && atop_left < below_right))
}

fn is_beside(
    left_transform: &Transform,
    left_box: &BoundingBox,
    right_transform: &Transform,
    right_box: &BoundingBox,
) -> bool {
    let left_bottom = left_transform.translation.y - left_box.0.y / 2.0;
    let left_top = left_transform.translation.y + left_box.0.y / 2.0;
    let left_right = left_transform.translation.x + left_box.0.x / 2.0;

    let right_top = right_transform.translation.y + right_box.0.y / 2.0;
    let right_bottom = right_transform.translation.y - right_box.0.y / 2.0;
    let right_left = right_transform.translation.x - right_box.0.x / 2.0;

    (left_right - right_left).abs() < std::f32::EPSILON
        && ((left_bottom <= right_bottom && right_bottom <= left_top)
            || (right_bottom <= left_bottom && left_bottom <= right_top))
}

fn reset_intrinsic_velocity(mut query: Query<&mut IntrinsicVelocity>) {
    for mut intrinsic_velocity in query.iter_mut() {
        *intrinsic_velocity = IntrinsicVelocity(None);
    }
}

fn reset_velocity(mut query: Query<&mut Velocity>) {
    for mut velocity in query.iter_mut() {
        *velocity = Velocity(None);
    }
}

fn build_adjacency_graph(
    mut adjacency_graph: ResMut<AdjacencyGraph>,

    left_query: Query<(Entity, &Transform, &BoundingBox), (Without<Escalator>)>,
    right_query: Query<(Entity, &Transform, &BoundingBox), ()>,
    atop_query: Query<(Entity, &Transform, &BoundingBox), Without<Step>>,
    bases_query: Query<(Entity, &Transform, &BoundingBox), Without<Escalator>>,
    steps: Query<(&Step, Entity)>,
) {
    // asymmetric for now b/c weirdness w/ elevator hitboxes
    let mut rights = HashMap::new();
    for (left_entity, left_transform, left_box) in left_query.iter() {
        for (right_entity, right_transform, right_box) in right_query.iter() {
            if is_beside(left_transform, left_box, right_transform, right_box) {
                let current_lefts = rights.entry(left_entity).or_insert_with(HashSet::new);
                current_lefts.insert(right_entity);
            }
        }
    }

    let mut lefts = HashMap::new();
    for (right_entity, right_transform, right_box) in right_query.iter() {
        for (left_entity, left_transform, left_box) in left_query.iter() {
            if is_beside(left_transform, left_box, right_transform, right_box) {
                let current_rights = lefts.entry(right_entity).or_insert_with(HashSet::new);
                current_rights.insert(left_entity);
            }
        }
    }

    let mut tops = HashMap::new();
    let mut bottoms = HashMap::new();

    for (atop_entity, atop_transform, atop_box) in atop_query.iter() {
        for (below_entity, below_transform, below_box) in bases_query.iter() {
            if is_atop(atop_transform, atop_box, below_transform, below_box) {
                let current_atops = tops.entry(below_entity).or_insert_with(HashSet::new);
                current_atops.insert(atop_entity);

                let current_bottoms = bottoms.entry(atop_entity).or_insert_with(HashSet::new);
                current_bottoms.insert(below_entity);
            }
        }
    }

    for (step, step_entity) in steps.iter() {
        let current_atops = tops.entry(step.escalator).or_insert_with(HashSet::new);
        current_atops.insert(step_entity);
    }

    *adjacency_graph = AdjacencyGraph {
        lefts,
        rights,
        tops,
        bottoms,
    };
}

fn falling_intrinsic_velocity(
    adjacency_graph: Res<AdjacencyGraph>,

    mut query: Query<
        (Entity, &mut IntrinsicVelocity),
        (Without<Player>, Without<Ground>, Without<Step>),
    >,
) {
    for (entity, mut intrinsic_velocity) in query.iter_mut() {
        match adjacency_graph.bottoms.get(&entity) {
            Some(bottoms) => {
                if bottoms.is_empty() {
                    *intrinsic_velocity = IntrinsicVelocity(Some(Propagation {
                        intrinsic: Some(Vec2::new(0.0, -1.0)),
                        ..Propagation::default()
                    }));
                }
            }
            None => {
                *intrinsic_velocity = IntrinsicVelocity(Some(Propagation {
                    intrinsic: Some(Vec2::new(0.0, -1.0)),
                    ..Propagation::default()
                }));
            }
        }

        if !adjacency_graph.bottoms.get(&entity).is_none() {}
    }
}

fn velocity_propagation(
    adjacency_graph: Res<AdjacencyGraph>,

    order_query: Query<(Entity, &Transform, &BoundingBox, &IntrinsicVelocity)>,

    mut velocities: Query<&mut Velocity>,

    grounds: Query<&Ground>,
    steps: Query<&Step>,
    ivs: Query<&IntrinsicVelocity>,
) {
    // order intrinsic velocities by y top

    let mut intrinsic_velocity_sources = vec![];

    for (entity, transform, bounding_box, intrinsic_velocity) in order_query.iter() {
        if let Some(intrinsic_velocity) = &intrinsic_velocity.0 {
            let top = transform.translation.y + bounding_box.0.y / 2.0;

            intrinsic_velocity_sources.push((entity, top, intrinsic_velocity));
        }
    }

    // don't need to sort here i think
    intrinsic_velocity_sources.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("sort velocity"));

    let mut propagation_results: HashMap<Entity, Propagation> = HashMap::new();

    for (entity, _top, intrinsic_velocity) in intrinsic_velocity_sources {
        let mut already_visited: HashSet<Entity> = HashSet::new();

        propagate_velocity(
            entity,
            &*adjacency_graph,
            &grounds,
            &steps,
            intrinsic_velocity.intrinsic.expect("asdf"),
            &ivs,
            &mut already_visited,
            &mut propagation_results,
        );
    }

    for (entity, propagation_result) in propagation_results.iter() {
        if let Err(e) = velocities.set(*entity, Velocity(Some(propagation_result.to_velocity()))) {
            eprint!("Error setting velocity: {:?}", e);
        }
    }
}

// set your IV
// trigger pushes + carries
fn propagate_velocity(
    entity: Entity,
    adjacency_graph: &AdjacencyGraph,
    grounds: &Query<&Ground>,
    steps: &Query<&Step>,
    mut intrinsic_velocity: Vec2,
    intrinsic_velocities: &Query<&IntrinsicVelocity>,

    already_visited: &mut HashSet<Entity>,
    propagation_results: &mut HashMap<Entity, Propagation>,
) {
    if grounds.get(entity).is_ok() {
        return;
    }

    if already_visited.contains(&entity) {
        return;
    }

    already_visited.insert(entity);

    // handle x first

    let mut x_blocked = false;

    if intrinsic_velocity.x < 0.0 {
        if let Some(left_entities) = adjacency_graph.lefts.get(&entity) {
            for left_entity in left_entities {
                x_blocked = x_blocked | test_left(*left_entity, adjacency_graph, grounds)
            }
        }
    }

    if intrinsic_velocity.x > 0.0 {
        if let Some(right_entities) = adjacency_graph.rights.get(&entity) {
            for right_entity in right_entities {
                x_blocked = x_blocked | test_right(*right_entity, adjacency_graph, grounds)
            }
        }
    }

    if x_blocked {
        intrinsic_velocity.x = 0.0;
    }

    let mut y_blocked = false;

    if intrinsic_velocity.y > 0.0 {
        if let Some(tops) = adjacency_graph.tops.get(&entity) {
            for top_entity in tops {
                y_blocked = y_blocked | test_up(*top_entity, adjacency_graph, grounds);
            }
        }
    }

    if intrinsic_velocity.y < 0.0 {
        if let Some(bottoms) = adjacency_graph.bottoms.get(&entity) {
            for bottom_entity in bottoms {
                y_blocked =
                y_blocked | test_down(*bottom_entity, adjacency_graph, grounds);
            }
        }
    }

    if y_blocked {
        intrinsic_velocity.y = 0.0
    }

    // handle self!

    if let Ok(step) = steps.get(entity) {
        let step_iv = intrinsic_velocities.get(entity).expect("step iv lookup");

        match propagation_results.get(&step.escalator) {
            Some(escalator_result) => {
                let escalator_result = escalator_result.clone();

                propagation_results.insert(
                    entity,
                    Propagation {
                        push: None,
                        // TODO: probably don't do this here
                        carry: Some(escalator_result.to_velocity()),
                        intrinsic: step_iv.0.clone().expect("asdf").intrinsic,
                    },
                );
            }
            None => {
                propagation_results.insert(entity, Propagation {
                    intrinsic: Some(intrinsic_velocity),
                    ..Propagation::default()
                });
            }
        }
    } else {
        // not a step!
        match propagation_results.entry(entity) {
            // someone propagated here already
            Entry::Occupied(mut existing_result) => {
                let existing_result = existing_result.get_mut();
                existing_result.intrinsic = Some(intrinsic_velocity);
            }
            Entry::Vacant(vacancy) => {
                vacancy.insert(Propagation {
                    intrinsic: Some(intrinsic_velocity),
                    ..Propagation::default()
                });
            }
        }
    }

    //push!

    // if let Some(tops) = adjacency_graph.tops.get(&entity) {
    //     for top_entity in tops {
    //         // carry!
    //         // propagate_velocity(
    //         //     *top_entity,
    //         //     propagation_velocity.clone(),
    //         //     adjacency_graph,
    //         //     grounds,
    //         //     steps,
    //         //     intrinsic_velocities,
    //         //     already_visited,
    //         //     propagation_results,
    //         // );
    //     }
    // }

    // handle 
}

// TODO: Merge test_* into a single fn (?)

fn test_left(entity: Entity, adjacency_graph: &AdjacencyGraph, grounds: &Query<&Ground>) -> bool {
    if grounds.get(entity).is_ok() {
        return true;
    }

    if let Some(left_entities) = adjacency_graph.lefts.get(&entity) {
        for left_entity in left_entities {
            if test_left(*left_entity, adjacency_graph, grounds) {
                return true;
            }
        }
    }

    false
}

fn test_right(entity: Entity, adjacency_graph: &AdjacencyGraph, grounds: &Query<&Ground>) -> bool {
    if grounds.get(entity).is_ok() {
        return true;
    }

    if let Some(right_entities) = adjacency_graph.rights.get(&entity) {
        for right_entity in right_entities {
            if test_right(*right_entity, adjacency_graph, grounds) {
                return true;
            }
        }
    }

    false
}

fn test_up(entity: Entity, adjacency_graph: &AdjacencyGraph, grounds: &Query<&Ground>) -> bool {
    if grounds.get(entity).is_ok() {
        return true;
    }

    if let Some(top_entities) = adjacency_graph.tops.get(&entity) {
        for top_entity in top_entities {
            if test_up(*top_entity, adjacency_graph, grounds) {
                return true;
            }
        }
    }

    false
}

fn test_down(entity: Entity, adjacency_graph: &AdjacencyGraph, grounds: &Query<&Ground>) -> bool {
    if grounds.get(entity).is_ok() {
        return true;
    }

    if let Some(bottom_entities) = adjacency_graph.bottoms.get(&entity) {
        for bottom_entity in bottom_entities {
            if test_down(*bottom_entity, adjacency_graph, grounds) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {

    use bevy::ecs::{FuncSystem, Stage};

    use super::*;

    fn helper<F>(commands_init: F, assertions: Vec<FuncSystem<()>>)
    where
        F: FnOnce(&mut Commands, &mut Resources) -> (),
    {
        let mut world = World::default();
        let mut resources = Resources::default();

        resources.insert(Input::<KeyCode>::default());
        resources.insert(AdjacencyGraph::default());

        let mut commands = Commands::default();

        commands.set_entity_reserver(world.get_entity_reserver());

        commands_init(&mut commands, &mut resources);
        commands.apply(&mut world, &mut resources);

        let mut stage = SystemStage::serial();

        stage
            // .add_system(bevy::input::system::exit_on_esc_system.system())
            // .add_system(framerate.system())
            // reset IV
            .add_system(reset_intrinsic_velocity.system())
            // build edge graph
            .add_system(build_adjacency_graph.system())
            // assign step IV
            .add_system(step_intrinsic_velocity.system())
            // assign player IV
            .add_system(player_intrinsic_velocity.system())
            // assign falling IV
            .add_system(falling_intrinsic_velocity.system())
            // propagation
            .add_system(reset_velocity.system())
            .add_system(velocity_propagation.system())
            // reset velocity
            // for each IV, in order of ascending y, propagate
            .add_system(update_position.system())
            .add_system(update_step_arm.system());

        for system in assertions {
            stage.add_system(system);
        }

        stage.initialize(&mut world, &mut resources);

        stage.run_once(&mut world, &mut resources)
    }

    #[test]
    fn player_falls_if_not_atop_anything() {
        helper(
            |commands, _resources| {
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(-250.0, 200.0, 1.0)),

                        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
                        ..Default::default()
                    })
                    .with(Player {})
                    .with(BoundingBox(Vec2::new(50.0, 50.0)))
                    .with(Velocity(None))
                    .with(IntrinsicVelocity(None));
            },
            vec![(|players: Query<(&Player, &Velocity)>| {
                for (_player, velocity) in players.iter() {
                    assert_eq!(velocity.0, Some(Vec2::new(0.0, -1.0)));
                }
            })
            .system()],
        )
    }

    #[test]
    fn player_doesnt_fall_if_atop_ground() {
        helper(
            |commands, _resources| {
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),

                        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
                        ..Default::default()
                    })
                    .with(Player {})
                    .with(BoundingBox(Vec2::new(50.0, 50.0)))
                    .with(Velocity(None))
                    .with(IntrinsicVelocity(None));

                let ground_box = Vec2::new(500.0, 50.0);
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                        sprite: Sprite::new(ground_box),
                        ..Default::default()
                    })
                    .with(Ground {})
                    .with(BoundingBox(ground_box))
                    .with(Velocity(None));
            },
            vec![(|players: Query<(&Player, &Velocity)>| {
                for (_player, velocity) in players.iter() {
                    assert_eq!(velocity.0, Some(Vec2::new(0.0, 0.0)));
                }
            })
            .system()],
        )
    }

    #[test]
    fn player_moves_left_when_a_is_pressed() {
        helper(
            |commands, resources| {
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),

                        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
                        ..Default::default()
                    })
                    .with(Player {})
                    .with(BoundingBox(Vec2::new(50.0, 50.0)))
                    .with(Velocity(None))
                    .with(IntrinsicVelocity(None));

                let ground_box = Vec2::new(500.0, 50.0);
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                        sprite: Sprite::new(ground_box),
                        ..Default::default()
                    })
                    .with(Ground {})
                    .with(BoundingBox(ground_box))
                    .with(Velocity(None));

                let mut input = Input::<KeyCode>::default();
                input.press(KeyCode::A);
                resources.insert(input)
            },
            vec![(|players: Query<(&Player, &Velocity)>| {
                for (_player, velocity) in players.iter() {
                    assert_eq!(velocity.0, Some(Vec2::new(-1.0, 0.0)));
                }
            })
            .system()],
        )
    }

    #[test]
    fn basic_propagation() {
        helper(
            |commands, resources| {
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),

                        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
                        ..Default::default()
                    })
                    .with(Player {})
                    .with(BoundingBox(Vec2::new(50.0, 50.0)))
                    .with(Velocity(None))
                    .with(IntrinsicVelocity(None));

                let ground_box = Vec2::new(500.0, 50.0);
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                        sprite: Sprite::new(ground_box),
                        ..Default::default()
                    })
                    .with(Ground {})
                    .with(BoundingBox(ground_box))
                    .with(Velocity(None));

                let crate_box = Vec2::new(50.0, 50.0);

                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(-50.0, 50.0, 1.0)),
                        sprite: Sprite::new(crate_box),
                        ..Default::default()
                    })
                    .with(Crate {})
                    .with(BoundingBox(crate_box))
                    .with(IntrinsicVelocity(None))
                    .with(Velocity(None));

                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(-100.0, 50.0, 1.0)),
                        sprite: Sprite::new(crate_box),
                        ..Default::default()
                    })
                    .with(Crate {})
                    .with(BoundingBox(crate_box))
                    .with(IntrinsicVelocity(None))
                    .with(Velocity(None));

                let mut input = Input::<KeyCode>::default();
                input.press(KeyCode::A);
                resources.insert(input)
            },
            vec![
                (|players: Query<(&Player, &Velocity)>| {
                    for (_player, velocity) in players.iter() {
                        assert_eq!(velocity.0, Some(Vec2::new(-1.0, 0.0)));
                    }
                })
                .system(),
                (|crates: Query<(&Crate, &Velocity)>| {
                    for (_crate, velocity) in crates.iter() {
                        assert_eq!(velocity.0, Some(Vec2::new(-1.0, 0.0)));
                    }
                })
                .system(),
            ],
        )
    }

    #[test]
    fn double_carry() {
        helper(
            |commands, resources| {
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),

                        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
                        ..Default::default()
                    })
                    .with(Player {})
                    .with(BoundingBox(Vec2::new(50.0, 50.0)))
                    .with(Velocity(None))
                    .with(IntrinsicVelocity(None));

                let ground_box = Vec2::new(500.0, 50.0);
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                        sprite: Sprite::new(ground_box),
                        ..Default::default()
                    })
                    .with(Ground {})
                    .with(BoundingBox(ground_box))
                    .with(Velocity(None));

                let crate_box = Vec2::new(50.0, 50.0);

                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(-50.0, 50.0, 1.0)),
                        sprite: Sprite::new(crate_box),
                        ..Default::default()
                    })
                    .with(Crate {})
                    .with(BoundingBox(crate_box))
                    .with(IntrinsicVelocity(None))
                    .with(Velocity(None));

                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(-25.0, 100.0, 1.0)),
                        sprite: Sprite::new(crate_box),
                        ..Default::default()
                    })
                    .with(Crate {})
                    .with(BoundingBox(crate_box))
                    .with(IntrinsicVelocity(None))
                    .with(Velocity(None));

                let mut input = Input::<KeyCode>::default();
                input.press(KeyCode::A);
                resources.insert(input)
            },
            vec![
                (|players: Query<(&Player, &Velocity)>| {
                    for (_player, velocity) in players.iter() {
                        assert_eq!(velocity.0, Some(Vec2::new(-1.0, 0.0)));
                    }
                })
                .system(),
                (|crates: Query<(&Crate, &Velocity)>| {
                    for (_crate, velocity) in crates.iter() {
                        assert_eq!(velocity.0, Some(Vec2::new(-1.0, 0.0)));
                    }
                })
                .system(),
            ],
        )
    }

    #[test]
    fn basic_blocking() {
        helper(
            |commands, resources| {
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),

                        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
                        ..Default::default()
                    })
                    .with(Player {})
                    .with(BoundingBox(Vec2::new(50.0, 50.0)))
                    .with(Velocity(None))
                    .with(IntrinsicVelocity(None));

                let ground_box = Vec2::new(500.0, 50.0);
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                        sprite: Sprite::new(ground_box),
                        ..Default::default()
                    })
                    .with(Ground {})
                    .with(BoundingBox(ground_box))
                    .with(Velocity(None));

                let crate_box = Vec2::new(50.0, 50.0);

                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(-50.0, 50.0, 1.0)),
                        sprite: Sprite::new(crate_box),
                        ..Default::default()
                    })
                    .with(Crate {})
                    .with(BoundingBox(crate_box))
                    .with(IntrinsicVelocity(None))
                    .with(Velocity(None));

                let ground_box = Vec2::new(50.0, 50.0);
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(-100.0, 0.0, 1.0)),
                        sprite: Sprite::new(ground_box),
                        ..Default::default()
                    })
                    .with(Ground {})
                    .with(BoundingBox(ground_box))
                    .with(Velocity(None));

                let mut input = Input::<KeyCode>::default();
                input.press(KeyCode::A);
                resources.insert(input);
            },
            vec![
                (|players: Query<(&Player, &Velocity)>| {
                    for (_player, velocity) in players.iter() {
                        assert_eq!(velocity.0, Some(Vec2::new(0.0, 0.0)));
                    }
                })
                .system(),
                (|crates: Query<(&Crate, &Velocity)>| {
                    for (_crate, velocity) in crates.iter() {
                        assert_eq!(velocity.0, None);
                    }
                })
                .system(),
            ],
        )
    }

    #[test]
    fn push_off_edge() {
        helper(
            |commands, resources| {
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),

                        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
                        ..Default::default()
                    })
                    .with(Player {})
                    .with(BoundingBox(Vec2::new(50.0, 50.0)))
                    .with(Velocity(None))
                    .with(IntrinsicVelocity(None));

                let ground_box = Vec2::new(50.0, 50.0);
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                        sprite: Sprite::new(ground_box),
                        ..Default::default()
                    })
                    .with(Ground {})
                    .with(BoundingBox(ground_box))
                    .with(Velocity(None));

                let crate_box = Vec2::new(50.0, 50.0);

                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(-50.0, 50.0, 1.0)),
                        sprite: Sprite::new(crate_box),
                        ..Default::default()
                    })
                    .with(Crate {})
                    .with(BoundingBox(crate_box))
                    .with(IntrinsicVelocity(None))
                    .with(Velocity(None));

                let mut input = Input::<KeyCode>::default();
                input.press(KeyCode::A);
                resources.insert(input)
            },
            vec![
                (|players: Query<(&Player, &Velocity)>| {
                    for (_player, velocity) in players.iter() {
                        assert_eq!(velocity.0, Some(Vec2::new(-1.0, 0.0)));
                    }
                })
                .system(),
                (|crates: Query<(&Crate, &Velocity)>| {
                    for (_crate, velocity) in crates.iter() {
                        assert_eq!(velocity.0, Some(Vec2::new(-1.0, -1.0)));
                    }
                })
                .system(),
            ],
        );
    }

    #[test]
    fn complex_fall() {
        struct A;
        struct B;
        struct C;

        helper(
            |commands, _resources| {
                let ground_box = Vec2::new(50.0, 50.0);
                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                        sprite: Sprite::new(ground_box),
                        ..Default::default()
                    })
                    .with(Ground {})
                    .with(BoundingBox(ground_box))
                    .with(Velocity(None));

                let crate_box = Vec2::new(50.0, 50.0);

                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
                        sprite: Sprite::new(crate_box),
                        ..Default::default()
                    })
                    .with(Crate {})
                    .with(BoundingBox(crate_box))
                    .with(IntrinsicVelocity(None))
                    .with(Velocity(None))
                    .with(A);

                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(-50.0, 50.0, 1.0)),
                        sprite: Sprite::new(crate_box),
                        ..Default::default()
                    })
                    .with(Crate {})
                    .with(BoundingBox(crate_box))
                    .with(IntrinsicVelocity(None))
                    .with(Velocity(None))
                    .with(B);

                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(-25.0, 100.0, 1.0)),
                        sprite: Sprite::new(crate_box),
                        ..Default::default()
                    })
                    .with(Crate {})
                    .with(BoundingBox(crate_box))
                    .with(IntrinsicVelocity(None))
                    .with(Velocity(None))
                    .with(C);
            },
            vec![
                (|crates: Query<(&A, &Velocity)>| {
                    for (_marker, velocity) in crates.iter() {
                        assert_eq!(*velocity, Velocity(None));
                    }
                })
                .system(),
                (|crates: Query<(&B, &Velocity)>| {
                    for (_marker, velocity) in crates.iter() {
                        assert_eq!(*velocity, Velocity(Some(Vec2::new(0.0, -1.0))));
                    }
                })
                .system(),
                (|crates: Query<(&C, &Velocity)>| {
                    for (_marker, velocity) in crates.iter() {
                        assert_eq!(*velocity, Velocity(Some(Vec2::new(0.0, 0.0))));
                    }
                })
                .system(),
            ],
        );
    }

    #[test]
    fn grounded_escalator_test() {
        helper(
            |commands, _resources| {
                let escalator_transform = Transform::from_translation(Vec3::zero());
                let escalator_box = Vec2::new(200.0, 200.0);

                let escalator = spawn_escalator(
                    commands,
                    Handle::default(),
                    escalator_transform,
                    escalator_box,
                );

                let step_box = Vec2::new(50.0, 50.0);
                for (step_transform, arm) in steps(escalator_transform, escalator_box, step_box)
                    .iter()
                    .take(1)
                {
                    spawn_step(
                        commands,
                        Handle::default(),
                        escalator,
                        *step_transform,
                        step_box,
                        arm.clone(),
                    );

                    commands
                        .spawn(SpriteBundle {
                            transform: *step_transform,
                            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
                            ..Default::default()
                        })
                        .with(BoundingBox(step_box.clone()))
                        .with(Step {
                            arm: arm.clone(),
                            escalator,
                        })
                        .with(Velocity(None))
                        .with(IntrinsicVelocity(None));
                }

                let ground_box = Vec2::new(300.0, 50.0);

                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite::new(ground_box),

                        transform: Transform::from_translation(Vec3::new(0.0, -125.0, 0.0)),
                        ..Default::default()
                    })
                    .with(BoundingBox(ground_box))
                    .with(Ground);
            },
            vec![(|steps: Query<(&Step, &Velocity)>| {
                for (_step, velocity) in steps.iter() {
                    assert_eq!(*velocity, Velocity(Some(Vec2::new(0.0, -1.0))));
                }
            })
            .system()],
        );
    }

    #[test]
    fn player_atop_escalator() {
        helper(
            |commands, _resources| {
                let escalator_transform = Transform::from_translation(Vec3::zero());
                let escalator_box = Vec2::new(200.0, 200.0);

                let escalator = spawn_escalator(
                    commands,
                    Handle::default(),
                    escalator_transform,
                    escalator_box,
                );

                let step_box = Vec2::new(50.0, 50.0);
                for (step_transform, arm) in steps(escalator_transform, escalator_box, step_box) {
                    spawn_step(
                        commands,
                        Handle::default(),
                        escalator,
                        step_transform,
                        step_box,
                        arm.clone(),
                    );
                }

                spawn_ground(
                    commands,
                    Handle::default(),
                    Vec2::new(300.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, -125.0, 0.0)),
                );

                spawn_player(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(25.0, 25.0, 0.0)),
                );
            },
            vec![(|steps: Query<(&Player, &Velocity)>| {
                for (_step, velocity) in steps.iter() {
                    assert_eq!(*velocity, Velocity(Some(Vec2::new(-1.0, 1.0))));
                }
            })
            .system()],
        );
    }

    #[test]
    fn player_atop_escalator_can_move_right() {
        helper(
            |commands, resources| {
                let escalator_transform = Transform::from_translation(Vec3::zero());
                let escalator_box = Vec2::new(200.0, 200.0);

                let escalator = spawn_escalator(
                    commands,
                    Handle::default(),
                    escalator_transform,
                    escalator_box,
                );

                let step_box = Vec2::new(50.0, 50.0);
                for (step_transform, arm) in steps(escalator_transform, escalator_box, step_box) {
                    spawn_step(
                        commands,
                        Handle::default(),
                        escalator,
                        step_transform,
                        step_box,
                        arm.clone(),
                    );
                }

                spawn_ground(
                    commands,
                    Handle::default(),
                    Vec2::new(300.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, -125.0, 0.0)),
                );

                spawn_player(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(25.0, 25.0, 0.0)),
                );

                let mut input = Input::<KeyCode>::default();
                input.press(KeyCode::D);
                resources.insert(input)
            },
            vec![(|steps: Query<(&Player, &Velocity)>| {
                for (_step, velocity) in steps.iter() {
                    assert_eq!(*velocity, Velocity(Some(Vec2::new(0.0, 1.0))));
                }
            })
            .system()],
        );
    }

    #[test]
    fn player_atop_escalator_cannot_move_left() {
        helper(
            |commands, resources| {
                let escalator_transform = Transform::from_translation(Vec3::zero());
                let escalator_box = Vec2::new(200.0, 200.0);

                let escalator = spawn_escalator(
                    commands,
                    Handle::default(),
                    escalator_transform,
                    escalator_box,
                );

                let step_box = Vec2::new(50.0, 50.0);
                for (step_transform, arm) in steps(escalator_transform, escalator_box, step_box) {
                    spawn_step(
                        commands,
                        Handle::default(),
                        escalator,
                        step_transform,
                        step_box,
                        arm.clone(),
                    );
                }

                spawn_ground(
                    commands,
                    Handle::default(),
                    Vec2::new(300.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, -125.0, 0.0)),
                );

                spawn_player(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(25.0, 25.0, 0.0)),
                );

                let mut input = Input::<KeyCode>::default();
                input.press(KeyCode::A);
                resources.insert(input)
            },
            vec![(|steps: Query<(&Player, &Velocity)>| {
                for (_step, velocity) in steps.iter() {
                    assert_eq!(*velocity, Velocity(Some(Vec2::new(-1.0, 1.0))));
                }
            })
            .system()],
        );
    }

    #[test]
    fn player_pushing_escalator() {
        struct A;

        helper(
            |commands, resources| {
                let escalator_transform = Transform::from_translation(Vec3::zero());
                let escalator_box = Vec2::new(200.0, 200.0);

                let escalator = spawn_escalator(
                    commands,
                    Handle::default(),
                    escalator_transform,
                    escalator_box,
                );

                let step_box = Vec2::new(50.0, 50.0);
                for (step_transform, arm) in steps(escalator_transform, escalator_box, step_box)
                    .iter()
                    .take(1)
                {
                    let a = spawn_step(
                        commands,
                        Handle::default(),
                        escalator,
                        *step_transform,
                        step_box,
                        arm.clone(),
                    );

                    commands.with(A);
                }

                spawn_ground(
                    commands,
                    Handle::default(),
                    Vec2::new(300.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, -125.0, 0.0)),
                );

                spawn_player(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(-125.0, -75.0, 0.0)),
                );

                let mut input = Input::<KeyCode>::default();
                input.press(KeyCode::D);
                resources.insert(input)
            },
            vec![(|steps: Query<(&A, &Velocity)>| {
                for (_step, velocity) in steps.iter() {
                    assert_eq!(*velocity, Velocity(Some(Vec2::new(1.0, -1.0))));
                }
            })
            .system()],
        );
    }

    #[test]
    fn falling_escalator() {
        helper(
            |commands, _resources| {
                let escalator_transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));

                let escalator_box = Vec2::new(200.0, 200.0);

                let escalator = commands
                    .spawn(SpriteSheetBundle {
                        sprite: TextureAtlasSprite {
                            color: Color::rgba(1.0, 1.0, 1.0, 0.5),
                            ..TextureAtlasSprite::default()
                        },

                        visible: Visible {
                            is_visible: true,
                            is_transparent: true,
                        },
                        transform: escalator_transform,
                        ..Default::default()
                    })
                    .with(Escalator {})
                    .with(Velocity(None))
                    .with(IntrinsicVelocity(None))
                    .with(BoundingBox(escalator_box.clone()))
                    .current_entity()
                    .expect("Parent");

                let step_box = Vec2::new(50.0, 50.0);
                for (step_transform, arm) in steps(escalator_transform, escalator_box, step_box)
                    .iter()
                    .take(1)
                {
                    commands
                        .spawn(SpriteBundle {
                            transform: *step_transform,
                            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
                            ..Default::default()
                        })
                        .with(step_box.clone())
                        .with(Step {
                            arm: arm.clone(),
                            escalator,
                        })
                        .with(Velocity(None))
                        .with(IntrinsicVelocity(None));
                }
            },
            vec![(|steps: Query<(&Step, &Velocity)>| {
                for (_step, velocity) in steps.iter() {
                    assert_eq!(*velocity, Velocity(Some(Vec2::new(0.0, -2.0))));
                }
            })
            .system()],
        );
    }
}
