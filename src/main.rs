use std::collections::{hash_map::Entry, HashMap, HashSet};

use bevy::{diagnostic::Diagnostics, prelude::*};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use nalgebra::{Isometry2, Point2, Vector2};
use parry2d::{query, shape::ConvexPolygon};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_resource(AdjacencyGraph::default())
        .add_startup_system(setup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(framerate.system())
        .add_system(update_step_arm.system())
        .add_system(update_step_track.system())
        // reset IV
        .add_system(reset_intrinsic_velocity.system())
        // build edge graph
        // .add_system(build_adjacency_graph.system())
        // assign step IV
        .add_system(step_intrinsic_velocity.system())
        // assign player IV
        .add_system(player_intrinsic_velocity.system())
        // assign falling IV
        .add_system(falling_intrinsic_velocity.system())
        // propagation
        // .add_system(reset_velocity.system())
        // .add_system(velocity_propagation.system())
        // reset velocity
        .add_system(process_collisions.system())
        // for each IV, in order of ascending y, propagate
        .add_system(update_position.system())
        .add_system(lines.system())
        .run();
}

#[allow(dead_code)]
fn framerate(diagnostics: Res<Diagnostics>) {
    if let Some(fps) = diagnostics.get(bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
        dbg!(fps.average());
    }
}

#[derive(Clone)]
struct BoundingBox(Vec2);

#[derive(Debug)]
struct ActiveBoundingBox;

struct Escalator;

struct Step {
    arm: Arm,
    escalator: Entity,
}

#[derive(Debug)]
struct Track {
    position: f32,
    length: f32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Arm {
    A,
    B,
    C,
    D,
}

#[derive(Clone, PartialEq, Debug)]
struct Velocity(Option<Vec2>);

#[derive(Debug)]
struct IntrinsicVelocity(Option<Propagation>);

#[derive(Clone, Debug)]
struct Propagation {
    left_x_bound: Option<f32>,
    right_x_bound: Option<f32>,
    push: Option<f32>,
    carry: Option<Vec2>,
    intrinsic: Option<Vec2>,
}

impl Default for Propagation {
    fn default() -> Self {
        Propagation {
            left_x_bound: None,
            right_x_bound: None,
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

        let mut result_x = push_x + carry.x + intrinsic.x;
        if let Some(left_x_bound) = self.left_x_bound {
            result_x = result_x.max(left_x_bound);
        }
        if let Some(right_x_bound) = self.right_x_bound {
            result_x = result_x.min(right_x_bound);
        }
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

fn t(x: f32, y: f32) -> Transform {
    Transform::from_translation(Vec3::new(x, y, 0.0))
}

#[allow(unused_variables)]
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
        spawn_ground(
            commands,
            ground_handle.clone_weak(),
            Vec2::new(600.0, 100.0),
            t(0.0, -100.0),
        );

        // spawn_crate(
        //     commands,
        //     crate_handle.clone_weak(),
        //     Vec2::new(50.0, 50.0),
        //     t(0.0, 200.0),
        // );

        // spawn_crate(
        //     commands,
        //     crate_handle.clone_weak(),
        //     Vec2::new(50.0, 50.0),
        //     t(0.0, 260.0),
        // );

        // spawn_crate(
        //     commands,
        //     crate_handle.clone_weak(),
        //     Vec2::new(50.0, 50.0),
        //     t(-100.0, 400.0),
        // );

        // spawn_crate(
        //     commands,
        //     crate_handle.clone_weak(),
        //     Vec2::new(50.0, 50.0),
        //     t(0.0, 400.0),
        // );

        // spawn_crate(
        //     commands,
        //     crate_handle.clone_weak(),
        //     Vec2::new(50.0, 50.0),
        //     t(50.0, 400.0),
        // );

        // spawn_crate(
        //     commands,
        //     crate_handle.clone_weak(),
        //     Vec2::new(50.0, 50.0),
        //     t(100.0, 400.0),
        // );

        spawn_player(
            commands,
            player_handle.clone_weak(),
            Vec2::new(50.0, 100.0),
            t(-100.0, 300.0),
        );

        let escalator_transform = t(50.0, 50.0);
        let escalator_box = Vec2::new(200.0, 200.0);
        let escalator = spawn_escalator(
            commands,
            escalator_handle.clone_weak(),
            escalator_transform,
            escalator_box,
        );

        let step_box = Vec2::new(50.0, 50.0);
        for (step_transform, arm, track_position, track_length) in
            steps(escalator_transform, escalator_box, step_box).iter()
        {
            dbg!("x");

            spawn_step(
                commands,
                step_handle.clone_weak(),
                escalator,
                *step_transform,
                step_box,
                arm.clone(),
                *track_position,
                *track_length,
            );
        }
    }
}

#[allow(dead_code)]
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
        .with(ActiveBoundingBox)
        .with(
            ConvexPolygon::from_convex_hull(&[
                Point2::new(-size.x / 2.0, size.y / 2.0),
                Point2::new(size.x / 2.0, -size.y / 2.0),
                Point2::new(-size.x / 2.0, -size.y / 2.0),
            ])
            .expect("polygon"),
        )
        .current_entity()
        .expect("escalator")
}

#[allow(dead_code)]
fn spawn_step(
    commands: &mut Commands,
    material: Handle<ColorMaterial>,
    escalator: Entity,
    transform: Transform,
    size: Vec2,
    arm: Arm,
    track_position: f32,
    track_length: f32,
) -> Entity {
    commands
        .spawn(SpriteBundle {
            material,
            transform,
            sprite: Sprite::new(size),
            ..Default::default()
        })
        .with(BoundingBox(size))
        .with(ActiveBoundingBox)
        .with(Step { arm, escalator })
        .with(Velocity(None))
        .with(IntrinsicVelocity(None))
        .with(Track {
            length: track_length,
            position: track_position,
        })
        .with(
            ConvexPolygon::from_convex_hull(&[
                Point2::new(-size.x / 2.0, size.y / 2.0),
                Point2::new(size.x / 2.0, size.y / 2.0),
                Point2::new(size.x / 2.0, -size.y / 2.0),
                Point2::new(-size.x / 2.0, -size.y / 2.0),
            ])
            .expect("poly"),
        )
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
        .with(ActiveBoundingBox)
        .with(Ground)
        .with(
            ConvexPolygon::from_convex_hull(&[
                Point2::new(-ground_box.x / 2.0, ground_box.y / 2.0),
                Point2::new(ground_box.x / 2.0, ground_box.y / 2.0),
                Point2::new(ground_box.x / 2.0, -ground_box.y / 2.0),
                Point2::new(-ground_box.x / 2.0, -ground_box.y / 2.0),
            ])
            .expect("polygon"),
        );
}

#[allow(dead_code)]
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
        .with(ActiveBoundingBox)
        .with(Velocity(None))
        .with(IntrinsicVelocity(None))
        .with(
            ConvexPolygon::from_convex_hull(&[
                Point2::new(-size.x / 2.0, size.y / 2.0),
                Point2::new(size.x / 2.0, size.y / 2.0),
                Point2::new(size.x / 2.0, -size.y / 2.0),
                Point2::new(-size.x / 2.0, -size.y / 2.0),
            ])
            .expect("poly"),
        );
}

#[allow(dead_code)]
fn spawn_crate(
    commands: &mut Commands,
    material: Handle<ColorMaterial>,
    size: Vec2,
    transform: Transform,
) -> Entity {
    commands
        .spawn(SpriteBundle {
            transform,
            sprite: Sprite::new(size),
            material,
            ..Default::default()
        })
        .with(Crate {})
        .with(BoundingBox(size))
        .with(ActiveBoundingBox)
        .with(IntrinsicVelocity(None))
        .with(Velocity(None))
        .current_entity()
        .expect("Spawned crate")
}

#[allow(dead_code)]
fn steps(
    escalator_transform: Transform,
    escalator_box: Vec2,
    step: Vec2,
) -> Vec<(Transform, Arm, f32, f32)> {
    let mut result = vec![];
    let n = (escalator_box.y / step.y) as i32;

    let track_length = (2.0 * (n as f32 - 1.0) + 2.0) * step.x;
    let mut track_position = 0.0;

    // A
    result.push((
        Transform::from_translation(Vec3::new(
            escalator_transform.translation.x - escalator_box.x / 2.0 + step.x / 2.0,
            escalator_transform.translation.y + escalator_box.y / 2.0 - step.y / 2.0,
            0.0,
        )),
        Arm::A,
        track_position,
        track_length,
    ));

    track_position += step.x;

    // B

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
            track_position,
            track_length,
        ));
        track_position += step.x;
    }

    // C
    result.push((
        Transform::from_translation(Vec3::new(
            escalator_transform.translation.x + escalator_box.x / 2.0 - 3.0 * step.y / 2.0,
            escalator_transform.translation.y - escalator_box.y / 2.0 + step.y / 2.0,
            0.0,
        )),
        Arm::C,
        track_position,
        track_length,
    ));
    track_position += step.x;

    // D
    for index in 0..n {
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
            track_position,
            track_length,
        ));
        track_position += step.x;
    }
    result
}

fn player_intrinsic_velocity(
    keyboard_input: Res<Input<KeyCode>>,
    adjacency_graph: Res<AdjacencyGraph>,
    mut query: Query<(&Player, Entity, &mut Velocity)>,
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

        *velocity = Velocity(Some(Vec2::new(x_velocity, y_velocity)));
    }
}

fn step_intrinsic_velocity(
    mut step_query: Query<(&Step, &BoundingBox, &Track, &Transform, &mut Velocity)>,
    escalator_query: Query<(&Escalator, &BoundingBox, &Transform)>,
) {
    for (step, step_box, track, step_transform, mut velocity) in step_query.iter_mut() {
        let (escalator, escalator_box, escalator_transform) = escalator_query
            .get(step.escalator)
            .expect("Step escalator lookup");

        let s = step_box.0.x;
        let N = escalator_box.0.x / s;

        let T_1 = s;
        let T_2 = s + (N - 1.) * s;
        let T_3 = 2. * s + (N - 1.) * s;

        let t = track.position;

        let target = escalator_transform.translation.truncate() + {
            if t < T_1 {
                Vec2::new(-(N - 3.0) * s / 2.0, (N - 1.0) * s / 2.0) + Vec2::new(-t, 0.0)
            } else if t < T_2 {
                Vec2::new(-(N - 1.0) * s / 2.0, (N - 1.0) * s / 2.0)
                    + Vec2::new(t - T_1, -(t - T_1))
            } else if t < T_3 {
                Vec2::new((N - 1.) * s / 2., -(N - 1.) * s / 2.) + Vec2::new(t - T_2, 0.0)
            } else {
                Vec2::new((N + 1.) * s / 2., -(N - 1.) * s / 2.) + Vec2::new(-(t - T_3), t - T_3)
            }
        };

        *velocity = Velocity(Some(target - step_transform.translation.truncate()));
    }
}

struct BoundingBoxTransform(Transform, BoundingBox);

impl BoundingBoxTransform {
    fn left(&self) -> f32 {
        self.0.translation.x - self.1 .0.x / 2.0
    }

    fn right(&self) -> f32 {
        self.0.translation.x + self.1 .0.x / 2.0
    }

    fn top(&self) -> f32 {
        self.0.translation.y + self.1 .0.y / 2.0
    }

    fn bottom(&self) -> f32 {
        self.0.translation.y - self.1 .0.y / 2.0
    }
}

fn process_collisions(
    q: Query<(Entity, &Transform, &ConvexPolygon)>,

    mut r: Query<&mut Velocity>,

    steps: Query<&Step>,
) {
    for (entity_a, xform_a, poly_a) in q.iter() {
        for (entity_b, xform_b, poly_b) in q.iter() {
            if entity_a >= entity_b {
                continue;
            }

            if let Ok(step_a) = steps.get(entity_a) {
                if step_a.escalator == entity_b {
                    continue;
                }

                if let Ok(step_b) = steps.get(entity_b) {
                    continue;
                }
            }

            if let Ok(step) = steps.get(entity_b) {
                if step.escalator == entity_a {
                    continue;
                }
            }

            // TODO: carry! 

            if let Some(contact) = collision(poly_a, &xform_a, poly_b, &xform_b) {
                if r.get_mut(entity_a).is_ok() && r.get_mut(entity_b).is_ok() {
                    let mut w = r.get_mut(entity_a).unwrap();
                    *w = Velocity(Some(
                        w.0.unwrap_or_else(Vec2::zero) + contact.normal1 * contact.dist / 2.0,
                    ));

                    let mut r = r.get_mut(entity_b).unwrap();
                    *r = Velocity(Some(
                        r.0.unwrap_or_else(Vec2::zero) + contact.normal2 * contact.dist / 2.0,
                    ));
                } else if let Ok(mut w) = r.get_mut(entity_a) {
                    *w = Velocity(Some(
                        w.0.unwrap_or_else(Vec2::zero) + contact.normal1 * contact.dist,
                    ));
                } else if let Ok(mut r) = r.get_mut(entity_b) {
                    *r = Velocity(Some(
                        r.0.unwrap_or_else(Vec2::zero) + contact.normal2 * contact.dist / 2.0,
                    ));
                } else {
                }

                // match (r.get_mut(entity_a), r.get_mut(entity_b)) {
                //     (Ok(mut w), Ok(mut r)) => {
                //         *w = Velocity(Some(w.0.unwrap_or_else(Vec2::zero) + contact.normal1 * contact.dist / 2.0));
                //         *r = Velocity(Some(r.0.unwrap_or_else(Vec2::zero) + contact.normal2 * contact.dist / 2.0));
                //     }
                //     (Ok(_), Err(_)) => {}
                //     (Err(_), Ok(_)) => {}
                //     (Err(_), Err(_)) => {}
                // }
            }
        }
    }
}

fn update_position(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    let delta_seconds = 10.0 * time.delta().as_secs_f32();

    for (maybe_velocity, mut transform) in query.iter_mut() {
        match maybe_velocity.0.to_owned() {
            Some(velocity) => {
                transform.translation.x += delta_seconds * velocity.x;
                transform.translation.y += delta_seconds * velocity.y;
            }
            None => {}
        }
    }
}

fn update_step_track(time: Res<Time>, mut steps: Query<(&Step, &mut Track)>) {
    let delta = 10.0 * time.delta_seconds();

    for (_step, mut track) in steps.iter_mut() {
        track.position = (track.position + delta) % track.length;
        // dbg!(track);
    }
}

fn update_step_arm(
    commands: &mut Commands,
    mut steps: Query<(Entity, &mut Step, &mut Track, &BoundingBox, &Transform)>,
    escalators: Query<(&Escalator, &BoundingBox, &Transform)>,
) {
    for (step_entity, mut step, track, step_box, step_transform) in steps.iter_mut() {
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
                if (step_bottom - (escalator_top - 2.0 * step_box.0.y)) < std::f32::EPSILON {
                    step.arm = Arm::B;
                    commands.remove_one::<ActiveBoundingBox>(step_entity);
                }
            }
            Arm::B => {
                if (step_bottom - escalator_bottom).abs() < std::f32::EPSILON {
                    step.arm = Arm::C;
                    commands.insert_one(step_entity, ActiveBoundingBox);
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
        && ((left_bottom <= right_bottom && right_bottom < left_top)
            || (right_bottom <= left_bottom && left_bottom < right_top))
}

fn reset_intrinsic_velocity(mut query: Query<&mut Velocity>) {
    for mut intrinsic_velocity in query.iter_mut() {
        *intrinsic_velocity = Velocity(None);
    }
}

fn reset_velocity(mut query: Query<&mut Velocity>) {
    for mut velocity in query.iter_mut() {
        *velocity = Velocity(None);
    }
}

fn build_adjacency_graph(
    mut adjacency_graph: ResMut<AdjacencyGraph>,

    left_query: Query<(Entity, &Transform, &BoundingBox), Without<Escalator>>,
    right_query: Query<(Entity, &Transform, &BoundingBox), ()>,
    atop_query: Query<(Entity, &Transform, &BoundingBox), Without<Step>>,
    bases_query: Query<(Entity, &Transform, &BoundingBox), Without<Escalator>>,
    steps: Query<(&Step, Entity)>,
) {
    // asymmetric for now b/c weirdness w/ elevator hitboxes
    let mut rights = HashMap::new();
    for (left_entity, left_transform, left_box) in left_query.iter() {
        for (right_entity, right_transform, right_box) in right_query.iter() {
            if steps.get(left_entity).is_ok() && steps.get(right_entity).is_ok() {
                continue;
            }

            if is_beside(left_transform, left_box, right_transform, right_box) {
                let current_lefts = rights.entry(left_entity).or_insert_with(HashSet::new);
                current_lefts.insert(right_entity);
            }
        }
    }

    let mut lefts = HashMap::new();
    for (right_entity, right_transform, right_box) in right_query.iter() {
        for (left_entity, left_transform, left_box) in left_query.iter() {
            if steps.get(left_entity).is_ok() && steps.get(right_entity).is_ok() {
                continue;
            }
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
            if steps.get(atop_entity).is_ok() && steps.get(below_entity).is_ok() {
                continue;
            }
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

        let current_bottoms = bottoms.entry(step_entity).or_insert_with(HashSet::new);
        current_bottoms.insert(step.escalator);
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

    mut query: Query<(Entity, &mut Velocity), (Without<Player>, Without<Ground>, Without<Step>)>,
) {
    for (entity, mut velocity) in query.iter_mut() {
        match adjacency_graph.bottoms.get(&entity) {
            Some(bottoms) => {
                if bottoms.is_empty() {
                    *velocity = Velocity(Some(Vec2::new(0.0, -1.0)));
                }
            }
            None => {
                *velocity = Velocity(Some(Vec2::new(0.0, -1.0)));
            }
        }
    }
}

fn velocity_propagation(
    adjacency_graph: Res<AdjacencyGraph>,

    order_query: Query<(Entity, &IntrinsicVelocity)>,

    mut velocities: Query<&mut Velocity>,

    grounds: Query<&Ground>,
    steps: Query<&Step>,
    ivs: Query<&IntrinsicVelocity>,
    actives: Query<&ActiveBoundingBox>,
) {
    // order intrinsic velocities by y top

    let mut propagation_results: HashMap<Entity, Propagation> = HashMap::new();

    let actives = &actives;

    for (entity, intrinsic_velocity) in order_query.iter() {
        if let Some(propagation) = intrinsic_velocity.0.clone() {
            let mut already_visited: HashSet<Entity> = HashSet::new();

            propagate_velocity(
                entity,
                &*adjacency_graph,
                &grounds,
                &steps,
                propagation.intrinsic.expect("asdf"),
                &ivs,
                &mut already_visited,
                &mut propagation_results,
                actives,
            );
        }
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
    actives: &Query<&ActiveBoundingBox>,
) {
    if grounds.get(entity).is_ok() {
        return;
    }

    if already_visited.contains(&entity) {
        return;
    }

    already_visited.insert(entity);

    // handle x first

    // bounds need to propagate too?
    let mut left_x_bound = None;

    if intrinsic_velocity.x < 0.0 {
        if let Some(left_entities) = adjacency_graph.lefts.get(&entity) {
            for left_entity in left_entities {
                match (
                    left_x_bound,
                    test_left(
                        *left_entity,
                        adjacency_graph,
                        grounds,
                        steps,
                        propagation_results,
                        intrinsic_velocities,
                        actives,
                    ),
                ) {
                    (None, None) => {}
                    (None, Some(new_bound)) => {
                        left_x_bound = Some(new_bound);
                    }
                    (Some(_), None) => {}
                    (Some(old_bound), Some(new_bound)) => {
                        left_x_bound = Some(old_bound.max(new_bound));
                    }
                }
            }
        }
    }

    let mut right_x_bound = None;

    if intrinsic_velocity.x > 0.0 {
        if let Some(right_entities) = adjacency_graph.rights.get(&entity) {
            for right_entity in right_entities {
                match (
                    right_x_bound,
                    test_right(
                        *right_entity,
                        adjacency_graph,
                        grounds,
                        steps,
                        propagation_results,
                        intrinsic_velocities,
                        actives,
                    ),
                ) {
                    (None, None) => {}
                    (None, Some(new_bound)) => {
                        right_x_bound = Some(new_bound);
                    }
                    (Some(_), None) => {}
                    (Some(old_bound), Some(new_bound)) => {
                        right_x_bound = Some(old_bound.min(new_bound));
                    }
                }
            }
        }
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
                y_blocked = y_blocked | test_down(*bottom_entity, adjacency_graph, grounds);
            }
        }
    }

    // TODO: clean this up; steps shouldn't get blocked b/c escalator is grounded
    if y_blocked && steps.get(entity).is_err() {
        intrinsic_velocity.y = 0.0;
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
                        left_x_bound: left_x_bound,
                        right_x_bound: right_x_bound,
                        // TODO: probably don't do this here
                        carry: Some(escalator_result.to_velocity()),
                        intrinsic: step_iv.0.clone().expect("asdf").intrinsic,
                        ..Propagation::default()
                    },
                );
            }
            None => {
                propagation_results.insert(
                    entity,
                    Propagation {
                        left_x_bound: left_x_bound,
                        right_x_bound: right_x_bound,
                        intrinsic: Some(intrinsic_velocity),
                        ..Propagation::default()
                    },
                );
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
                    left_x_bound: left_x_bound,
                    right_x_bound: right_x_bound,
                    intrinsic: Some(intrinsic_velocity),
                    ..Propagation::default()
                });
            }
        }
    }

    if actives.get(entity).is_ok() {
        if intrinsic_velocity.x > 0.0 {
            if let Some(rights) = adjacency_graph.rights.get(&entity) {
                for right_entity in rights {
                    x_push(
                        intrinsic_velocity.x,
                        *right_entity,
                        adjacency_graph,
                        already_visited,
                        propagation_results,
                        grounds,
                        steps,
                        &actives,
                        intrinsic_velocities,
                    );
                }
            }
        }

        if intrinsic_velocity.x < 0.0 {
            if let Some(lefts) = adjacency_graph.lefts.get(&entity) {
                for left_entity in lefts {
                    x_push(
                        intrinsic_velocity.x,
                        *left_entity,
                        adjacency_graph,
                        already_visited,
                        propagation_results,
                        grounds,
                        steps,
                        &actives,
                        intrinsic_velocities,
                    );
                }
            }
        }

        if let Some(tops) = adjacency_graph.tops.get(&entity) {
            for top_entity in tops {
                carry(
                    intrinsic_velocity,
                    *top_entity,
                    adjacency_graph,
                    already_visited,
                    propagation_results,
                    grounds,
                    steps,
                    intrinsic_velocities,
                    actives,
                );
            }
        }
    }
}

// set x velocity, possibly checking for test (?)
// carry
fn x_push(
    push_x: f32,
    entity: Entity,
    adjacency_graph: &AdjacencyGraph,
    already_visited: &mut HashSet<Entity>,
    propagation_results: &mut HashMap<Entity, Propagation>,
    grounds: &Query<&Ground>,
    steps: &Query<&Step>,
    actives: &Query<&ActiveBoundingBox>,
    ivs: &Query<&IntrinsicVelocity>,
) {
    if grounds.get(entity).is_ok() {
        return;
    }
    if already_visited.contains(&entity) {
        return;
    }
    already_visited.insert(entity);

    let mut left_x_bound = None;

    if push_x < 0.0 {
        if let Some(left_entities) = adjacency_graph.lefts.get(&entity) {
            for left_entity in left_entities {
                match (
                    left_x_bound,
                    test_left(
                        *left_entity,
                        adjacency_graph,
                        grounds,
                        steps,
                        propagation_results,
                        ivs,
                        actives,
                    ),
                ) {
                    (None, None) => {}
                    (None, Some(new_bound)) => {
                        left_x_bound = Some(new_bound);
                    }
                    (Some(_), None) => {}
                    (Some(old_bound), Some(new_bound)) => {
                        left_x_bound = Some(old_bound.max(new_bound));
                    }
                }
            }
        }
    }

    let mut right_x_bound = None;

    if push_x > 0.0 {
        if let Some(right_entities) = adjacency_graph.rights.get(&entity) {
            for right_entity in right_entities {
                match (
                    right_x_bound,
                    test_right(
                        *right_entity,
                        adjacency_graph,
                        grounds,
                        steps,
                        propagation_results,
                        ivs,
                        actives,
                    ),
                ) {
                    (None, None) => {}
                    (None, Some(new_bound)) => {
                        right_x_bound = Some(new_bound);
                    }
                    (Some(_), None) => {}
                    (Some(old_bound), Some(new_bound)) => {
                        right_x_bound = Some(old_bound.min(new_bound));
                    }
                }
            }
        }
    }

    // TODO: test?
    // TODO: might max_abs persist stale events? probably
    match propagation_results.entry(entity) {
        Entry::Occupied(mut existing_result) => {
            let existing_result = existing_result.get_mut();
            existing_result.left_x_bound = left_x_bound;
            existing_result.right_x_bound = right_x_bound;
            match existing_result.push {
                Some(existing_push) => {
                    existing_result.push = Some(max_abs(existing_push, push_x));
                }
                None => {
                    existing_result.push = Some(push_x);
                }
            }
        }
        Entry::Vacant(vacancy) => {
            vacancy.insert(Propagation {
                left_x_bound,
                right_x_bound,
                push: Some(push_x),
                ..Propagation::default()
            });
        }
    }

    if actives.get(entity).is_ok() {
        if push_x > 0.0 {
            if let Some(rights) = adjacency_graph.rights.get(&entity) {
                for right_entity in rights {
                    x_push(
                        push_x,
                        *right_entity,
                        adjacency_graph,
                        already_visited,
                        propagation_results,
                        grounds,
                        steps,
                        actives,
                        ivs,
                    );
                }
            }
        }

        if push_x < 0.0 {
            if let Some(lefts) = adjacency_graph.lefts.get(&entity) {
                for left_entity in lefts {
                    x_push(
                        push_x,
                        *left_entity,
                        adjacency_graph,
                        already_visited,
                        propagation_results,
                        grounds,
                        steps,
                        actives,
                        ivs,
                    );
                }
            }
        }

        // TODO: carry
        if let Some(atops) = adjacency_graph.tops.get(&entity) {
            for atop_entity in atops {
                carry(
                    Vec2::new(push_x, 0.0),
                    *atop_entity,
                    adjacency_graph,
                    already_visited,
                    propagation_results,
                    grounds,
                    steps,
                    ivs,
                    actives,
                )
            }
        }
    }
}

fn carry(
    carry_velocity: Vec2,
    entity: Entity,
    adjacency_graph: &AdjacencyGraph,
    already_visited: &mut HashSet<Entity>,
    propagation_results: &mut HashMap<Entity, Propagation>,
    grounds: &Query<&Ground>,
    steps: &Query<&Step>,
    ivs: &Query<&IntrinsicVelocity>,
    actives: &Query<&ActiveBoundingBox>,
) {
    if grounds.get(entity).is_ok() {
        return;
    }
    if already_visited.contains(&entity) {
        return;
    }
    already_visited.insert(entity);
    // query across bottoms... RHS won't ever get a propagation result tho?
    // default to 0 for now

    let mut max_bottom_velocity: Option<Vec2> = None;

    if let Some(bottom_entities) = adjacency_graph.bottoms.get(&entity) {
        for bottom_entity in bottom_entities {
            match (max_bottom_velocity, propagation_results.get(bottom_entity)) {
                (None, None) => {
                    // found first bottom
                    // bottom seems to be still
                    max_bottom_velocity = Some(Vec2::zero());
                }
                (None, Some(new_bottom)) => {
                    // found first bottom
                    max_bottom_velocity = Some(new_bottom.to_velocity());
                }
                (Some(found_velocity), None) => {
                    // we already found something; is it falling?
                    // if so, zero out
                    if found_velocity.y < 0.0 {
                        max_bottom_velocity = Some(Vec2::zero());
                    }
                }
                (Some(found_velocity), Some(new_bottom)) => {
                    if found_velocity.y < new_bottom.to_velocity().y {
                        max_bottom_velocity = Some(new_bottom.to_velocity());
                    }
                }
            }
        }
    }

    let mut left_x_bound = None;

    if let Some(left_entities) = adjacency_graph.lefts.get(&entity) {
        for left_entity in left_entities {
            match (
                left_x_bound,
                test_left(
                    *left_entity,
                    adjacency_graph,
                    grounds,
                    steps,
                    propagation_results,
                    ivs,
                    actives,
                ),
            ) {
                (None, None) => {}
                (None, Some(new_bound)) => {
                    left_x_bound = Some(new_bound);
                }
                (Some(_), None) => {}
                (Some(old_bound), Some(new_bound)) => {
                    left_x_bound = Some(old_bound.max(new_bound));
                }
            }
        }
    }

    let mut right_x_bound = None;
    if let Some(right_entities) = adjacency_graph.rights.get(&entity) {
        for right_entity in right_entities {
            match (
                right_x_bound,
                test_right(
                    *right_entity,
                    adjacency_graph,
                    grounds,
                    steps,
                    propagation_results,
                    ivs,
                    actives,
                ),
            ) {
                (None, None) => {}
                (None, Some(new_bound)) => {
                    right_x_bound = Some(new_bound);
                }
                (Some(_), None) => {}
                (Some(old_bound), Some(new_bound)) => {
                    right_x_bound = Some(old_bound.min(new_bound));
                }
            }
        }
    }

    // TODO: test?
    match propagation_results.entry(entity) {
        Entry::Occupied(mut existing_result) => {
            let existing_result = existing_result.get_mut();
            existing_result.left_x_bound = left_x_bound;
            existing_result.right_x_bound = right_x_bound;
            existing_result.carry = max_bottom_velocity;
        }
        Entry::Vacant(vacancy) => {
            vacancy.insert(Propagation {
                left_x_bound,
                right_x_bound,
                carry: max_bottom_velocity,
                ..Propagation::default()
            });
        }
    }
}

fn max_abs(a: f32, b: f32) -> f32 {
    if a.abs() > b.abs() {
        return a;
    }
    b
}

// TODO: Merge test_* into a single fn (?)
// Want to find the max right velocity of a step/ground
fn test_left(
    entity: Entity,
    adjacency_graph: &AdjacencyGraph,
    grounds: &Query<&Ground>,
    steps: &Query<&Step>,
    propagation_results: &mut HashMap<Entity, Propagation>,
    ivs: &Query<&IntrinsicVelocity>,
    actives: &Query<&ActiveBoundingBox>,
) -> Option<f32> {
    if grounds.get(entity).is_ok() {
        return Some(0.0);
    }
    if actives.get(entity).is_err() {
        return None;
    }

    if steps.get(entity).is_ok() {
        match propagation_results.get(&entity) {
            Some(propagation) => {
                return Some(propagation.to_velocity().x);
            }
            None => {
                return ivs
                    .get(entity)
                    .expect("step iv lookup")
                    .0
                    .clone()
                    .expect("step IV")
                    .intrinsic
                    .map(|intrinsic| intrinsic.x)
            }
        }
    }

    let mut max_x_velocity = None;

    if let Some(left_entities) = adjacency_graph.lefts.get(&entity) {
        for left_entity in left_entities {
            match (
                max_x_velocity,
                test_left(
                    *left_entity,
                    adjacency_graph,
                    grounds,
                    steps,
                    propagation_results,
                    ivs,
                    actives,
                ),
            ) {
                (None, None) => {}
                (None, Some(new_bound)) => {
                    max_x_velocity = Some(new_bound);
                }
                (Some(_), None) => {}
                (Some(old_bound), Some(new_bound)) => {
                    max_x_velocity = Some(old_bound.max(new_bound));
                }
            }
        }
    }

    match propagation_results.entry(entity) {
        Entry::Occupied(mut old_entry) => {
            let old_entry = old_entry.get_mut();
            *old_entry = Propagation {
                left_x_bound: max_x_velocity,
                ..*old_entry
            }
        }
        Entry::Vacant(empty) => {
            empty.insert(Propagation {
                left_x_bound: max_x_velocity,
                ..Propagation::default()
            });
        }
    }

    max_x_velocity
}

fn test_right(
    entity: Entity,
    adjacency_graph: &AdjacencyGraph,
    grounds: &Query<&Ground>,
    steps: &Query<&Step>,
    propagation_results: &mut HashMap<Entity, Propagation>,
    ivs: &Query<&IntrinsicVelocity>,
    actives: &Query<&ActiveBoundingBox>,
) -> Option<f32> {
    if grounds.get(entity).is_ok() {
        return Some(0.0);
    }

    if actives.get(entity).is_err() {
        return None;
    }

    if steps.get(entity).is_ok() {
        match propagation_results.get(&entity) {
            Some(propagation) => {
                return Some(propagation.to_velocity().x);
            }
            None => {
                return ivs
                    .get(entity)
                    .expect("step iv lookup")
                    .0
                    .clone()
                    .expect("step IV")
                    .intrinsic
                    .map(|intrinsic| intrinsic.x)
            }
        }
    }

    let mut min_x_velocity = None;

    if let Some(right_entities) = adjacency_graph.rights.get(&entity) {
        for right_entity in right_entities {
            match (
                min_x_velocity,
                test_right(
                    *right_entity,
                    adjacency_graph,
                    grounds,
                    steps,
                    propagation_results,
                    ivs,
                    actives,
                ),
            ) {
                (None, None) => {}
                (None, Some(new_bound)) => {
                    min_x_velocity = Some(new_bound);
                }
                (Some(_), None) => {}
                (Some(old_bound), Some(new_bound)) => {
                    min_x_velocity = Some(old_bound.min(new_bound));
                }
            }
        }
    }

    match propagation_results.entry(entity) {
        Entry::Occupied(mut old_entry) => {
            let old_entry = old_entry.get_mut();
            *old_entry = Propagation {
                right_x_bound: min_x_velocity,
                ..*old_entry
            }
        }
        Entry::Vacant(empty) => {
            empty.insert(Propagation {
                right_x_bound: min_x_velocity,
                ..Propagation::default()
            });
        }
    }

    min_x_velocity
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

fn lines(mut lines: ResMut<DebugLines>, q: Query<(&Transform, &ConvexPolygon)>) {
    for (xform, polygon) in q.iter() {
        for (point1, point2) in polygon.points().iter().skip(1).zip(polygon.points()) {
            let start = Vec3::new(
                point1.x + xform.translation.x,
                point1.y + xform.translation.y,
                0.0,
            );

            let end = Vec3::new(
                point2.x + xform.translation.x,
                point2.y + xform.translation.y,
                0.0,
            );

            lines.line(start, end, 1.);
        }

        if let Some(point1) = polygon.points().first() {
            if let Some(point2) = polygon.points().last() {
                let start = Vec3::new(
                    point1.x + xform.translation.x,
                    point1.y + xform.translation.y,
                    0.0,
                );

                let end = Vec3::new(
                    point2.x + xform.translation.x,
                    point2.y + xform.translation.y,
                    0.0,
                );

                lines.line(start, end, 1.);
            }
        }
    }
}

struct BevyCollision {
    normal1: Vec2,
    normal2: Vec2,
    dist: f32,
}

fn collision(
    poly1: &ConvexPolygon,
    xform1: &Transform,
    poly2: &ConvexPolygon,
    xform2: &Transform,
) -> Option<BevyCollision> {
    let p1 = Vector2::new(xform1.translation.x, xform1.translation.y);
    let i1 = Isometry2::new(p1, 0.0);

    let p2 = Vector2::new(xform2.translation.x, xform2.translation.y);
    let i2 = Isometry2::new(p2, 0.0);

    query::contact(&i1, poly1, &i2, poly2, 0.1)
        .map(|contact| {
            contact.map(|contact| {
                if contact.dist >= 0.0 {
                    return None;
                }

                return Some(BevyCollision {
                    normal1: Vec2::new(contact.normal1.x, contact.normal1.y),
                    normal2: Vec2::new(contact.normal2.x, contact.normal2.y),
                    dist: contact.dist,
                });
            })
        })
        .ok()
        .flatten()
        .flatten()
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
                spawn_player(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::default(),
                );
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
                spawn_player(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
                );

                spawn_ground(
                    commands,
                    Handle::default(),
                    Vec2::new(500.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                );
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
                spawn_player(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
                );

                spawn_ground(
                    commands,
                    Handle::default(),
                    Vec2::new(500.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                );

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
                spawn_player(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
                );
                spawn_ground(
                    commands,
                    Handle::default(),
                    Vec2::new(500.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                );
                spawn_crate(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(-50.0, 50.0, 1.0)),
                );
                spawn_crate(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(-100.0, 50.0, 1.0)),
                );

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
                spawn_player(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
                );

                spawn_ground(
                    commands,
                    Handle::default(),
                    Vec2::new(500.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                );

                spawn_crate(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(-50.0, 50.0, 1.0)),
                );

                spawn_crate(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(-25.0, 100.0, 1.0)),
                );

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
    fn basic_blocking_left() {
        helper(
            |commands, resources| {
                spawn_player(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
                );
                spawn_ground(
                    commands,
                    Handle::default(),
                    Vec2::new(500.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                );
                spawn_crate(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(-50.0, 50.0, 1.0)),
                );
                spawn_ground(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(-100.0, 50.0, 1.0)),
                );

                let mut input = Input::<KeyCode>::default();
                input.press(KeyCode::A);
                resources.insert(input);
            },
            vec![
                (|players: Query<(&Player, &Velocity)>| {
                    for (_player, velocity) in players.iter() {
                        assert_eq!(velocity.0, Some(Vec2::zero()));
                    }
                })
                .system(),
                (|crates: Query<(&Crate, &Velocity)>| {
                    for (_crate, velocity) in crates.iter() {
                        assert_eq!(velocity.0, Some(Vec2::zero()));
                    }
                })
                .system(),
            ],
        )
    }

    #[test]
    fn basic_blocking_right() {
        helper(
            |commands, resources| {
                spawn_player(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
                );
                spawn_ground(
                    commands,
                    Handle::default(),
                    Vec2::new(500.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                );
                spawn_crate(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(50.0, 50.0, 1.0)),
                );
                spawn_ground(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(100.0, 50.0, 1.0)),
                );

                let mut input = Input::<KeyCode>::default();
                input.press(KeyCode::D);
                resources.insert(input);
            },
            vec![
                (|players: Query<(&Player, &Velocity)>| {
                    for (_player, velocity) in players.iter() {
                        assert_eq!(velocity.0, Some(Vec2::zero()));
                    }
                })
                .system(),
                (|crates: Query<(&Crate, &Velocity)>| {
                    for (_crate, velocity) in crates.iter() {
                        assert_eq!(velocity.0, Some(Vec2::zero()));
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
                spawn_player(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
                );

                spawn_ground(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                );

                spawn_crate(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(-50.0, 50.0, 1.0)),
                );

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
                spawn_ground(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                );

                spawn_crate(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
                );
                commands.with(A);

                spawn_crate(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(-50.0, 50.0, 1.0)),
                );
                commands.with(B);

                spawn_crate(
                    commands,
                    Handle::default(),
                    Vec2::new(50.0, 50.0),
                    Transform::from_translation(Vec3::new(-25.0, 100.0, 1.0)),
                );
                commands.with(C);
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

    // #[test]
    // fn grounded_escalator_test() {
    //     helper(
    //         |commands, _resources| {
    //             let escalator_transform = Transform::from_translation(Vec3::zero());
    //             let escalator_box = Vec2::new(200.0, 200.0);

    //             let escalator = spawn_escalator(
    //                 commands,
    //                 Handle::default(),
    //                 escalator_transform,
    //                 escalator_box,
    //             );

    //             let step_box = Vec2::new(50.0, 50.0);
    //             for (step_transform, arm) in steps(escalator_transform, escalator_box, step_box)
    //                 .iter()
    //                 .take(1)
    //             {
    //                 spawn_step(
    //                     commands,
    //                     Handle::default(),
    //                     escalator,
    //                     *step_transform,
    //                     step_box,
    //                     arm.clone(),
    //                 );
    //             }

    //             let ground_box = Vec2::new(300.0, 50.0);

    //             spawn_ground(
    //                 commands,
    //                 Handle::default(),
    //                 ground_box,
    //                 Transform::from_translation(Vec3::new(0.0, -125.0, 0.0)),
    //             );
    //         },
    //         vec![(|steps: Query<(&Step, &Velocity)>| {
    //             for (_step, velocity) in steps.iter() {
    //                 assert_eq!(*velocity, Velocity(Some(Vec2::new(0.0, -1.0))));
    //             }
    //         })
    //         .system()],
    //     );
    // }

    // #[test]
    // fn player_atop_escalator() {
    //     helper(
    //         |commands, _resources| {
    //             let escalator_transform = Transform::from_translation(Vec3::zero());
    //             let escalator_box = Vec2::new(200.0, 200.0);

    //             let escalator = spawn_escalator(
    //                 commands,
    //                 Handle::default(),
    //                 escalator_transform,
    //                 escalator_box,
    //             );

    //             let step_box = Vec2::new(50.0, 50.0);
    //             for (step_transform, arm) in steps(escalator_transform, escalator_box, step_box) {
    //                 spawn_step(
    //                     commands,
    //                     Handle::default(),
    //                     escalator,
    //                     step_transform,
    //                     step_box,
    //                     arm.clone(),
    //                 );
    //             }

    //             spawn_ground(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(300.0, 50.0),
    //                 Transform::from_translation(Vec3::new(0.0, -125.0, 0.0)),
    //             );

    //             spawn_player(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(50.0, 50.0),
    //                 Transform::from_translation(Vec3::new(25.0, 25.0, 0.0)),
    //             );
    //         },
    //         vec![(|steps: Query<(&Player, &Velocity)>| {
    //             for (_step, velocity) in steps.iter() {
    //                 assert_eq!(*velocity, Velocity(Some(Vec2::new(-1.0, 1.0))));
    //             }
    //         })
    //         .system()],
    //     );
    // }

    // #[test]
    // fn player_atop_escalator_can_move_right() {
    //     helper(
    //         |commands, resources| {
    //             let escalator_transform = Transform::from_translation(Vec3::zero());
    //             let escalator_box = Vec2::new(200.0, 200.0);

    //             let escalator = spawn_escalator(
    //                 commands,
    //                 Handle::default(),
    //                 escalator_transform,
    //                 escalator_box,
    //             );

    //             let step_box = Vec2::new(50.0, 50.0);
    //             for (step_transform, arm) in steps(escalator_transform, escalator_box, step_box) {
    //                 spawn_step(
    //                     commands,
    //                     Handle::default(),
    //                     escalator,
    //                     step_transform,
    //                     step_box,
    //                     arm.clone(),
    //                 );
    //             }

    //             spawn_ground(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(300.0, 50.0),
    //                 Transform::from_translation(Vec3::new(0.0, -125.0, 0.0)),
    //             );

    //             spawn_player(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(50.0, 50.0),
    //                 Transform::from_translation(Vec3::new(25.0, 25.0, 0.0)),
    //             );

    //             let mut input = Input::<KeyCode>::default();
    //             input.press(KeyCode::D);
    //             resources.insert(input)
    //         },
    //         vec![(|steps: Query<(&Player, &Velocity)>| {
    //             for (_step, velocity) in steps.iter() {
    //                 assert_eq!(*velocity, Velocity(Some(Vec2::new(0.0, 1.0))));
    //             }
    //         })
    //         .system()],
    //     );
    // }

    // #[test]
    // fn player_atop_escalator_cannot_move_left() {
    //     helper(
    //         |commands, resources| {
    //             let escalator_transform = Transform::from_translation(Vec3::zero());
    //             let escalator_box = Vec2::new(200.0, 200.0);

    //             let escalator = spawn_escalator(
    //                 commands,
    //                 Handle::default(),
    //                 escalator_transform,
    //                 escalator_box,
    //             );

    //             let step_box = Vec2::new(50.0, 50.0);
    //             for (step_transform, arm) in steps(escalator_transform, escalator_box, step_box) {
    //                 spawn_step(
    //                     commands,
    //                     Handle::default(),
    //                     escalator,
    //                     step_transform,
    //                     step_box,
    //                     arm.clone(),
    //                 );
    //             }

    //             spawn_ground(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(300.0, 50.0),
    //                 Transform::from_translation(Vec3::new(0.0, -125.0, 0.0)),
    //             );

    //             spawn_player(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(50.0, 50.0),
    //                 Transform::from_translation(Vec3::new(25.0, 25.0, 0.0)),
    //             );

    //             let mut input = Input::<KeyCode>::default();
    //             input.press(KeyCode::A);
    //             resources.insert(input)
    //         },
    //         vec![(|steps: Query<(&Player, &Velocity)>| {
    //             for (_step, velocity) in steps.iter() {
    //                 assert_eq!(*velocity, Velocity(Some(Vec2::new(-1.0, 1.0))));
    //             }
    //         })
    //         .system()],
    //     );
    // }

    // #[test]
    // fn player_pushing_escalator() {
    //     struct A;

    //     helper(
    //         |commands, resources| {
    //             let escalator_transform = Transform::from_translation(Vec3::zero());
    //             let escalator_box = Vec2::new(200.0, 200.0);

    //             let escalator = spawn_escalator(
    //                 commands,
    //                 Handle::default(),
    //                 escalator_transform,
    //                 escalator_box,
    //             );

    //             let step_box = Vec2::new(50.0, 50.0);
    //             for (step_transform, arm) in steps(escalator_transform, escalator_box, step_box)
    //                 .iter()
    //                 .take(1)
    //             {
    //                 spawn_step(
    //                     commands,
    //                     Handle::default(),
    //                     escalator,
    //                     *step_transform,
    //                     step_box,
    //                     arm.clone(),
    //                 );

    //                 commands.with(A);
    //             }

    //             spawn_ground(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(300.0, 50.0),
    //                 Transform::from_translation(Vec3::new(0.0, -125.0, 0.0)),
    //             );

    //             spawn_player(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(50.0, 50.0),
    //                 Transform::from_translation(Vec3::new(-125.0, -75.0, 0.0)),
    //             );

    //             let mut input = Input::<KeyCode>::default();
    //             input.press(KeyCode::D);
    //             resources.insert(input)
    //         },
    //         vec![(|steps: Query<(&A, &Velocity)>| {
    //             for (_step, velocity) in steps.iter() {
    //                 assert_eq!(*velocity, Velocity(Some(Vec2::new(1.0, -1.0))));
    //             }
    //         })
    //         .system()],
    //     );
    // }

    // #[test]
    // fn falling_escalator() {
    //     helper(
    //         |commands, _resources| {
    //             let escalator_transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));

    //             let escalator_box = Vec2::new(200.0, 200.0);

    //             let escalator = spawn_escalator(
    //                 commands,
    //                 Handle::default(),
    //                 escalator_transform,
    //                 escalator_box,
    //             );

    //             let step_box = Vec2::new(50.0, 50.0);
    //             for (step_transform, arm) in steps(escalator_transform, escalator_box, step_box)
    //                 .iter()
    //                 .take(1)
    //             {
    //                 spawn_step(
    //                     commands,
    //                     Handle::default(),
    //                     escalator,
    //                     *step_transform,
    //                     step_box,
    //                     arm.clone(),
    //                 );
    //             }
    //         },
    //         vec![(|steps: Query<(&Step, &Velocity)>| {
    //             for (_step, velocity) in steps.iter() {
    //                 assert_eq!(*velocity, Velocity(Some(Vec2::new(0.0, -2.0))));
    //             }
    //         })
    //         .system()],
    //     );
    // }

    // #[test]
    // fn falling_escalator_2() {
    //     helper(
    //         |commands, _resources| {
    //             let escalator_transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));

    //             let escalator_box = Vec2::new(200.0, 200.0);

    //             let escalator = spawn_escalator(
    //                 commands,
    //                 Handle::default(),
    //                 escalator_transform,
    //                 escalator_box,
    //             );

    //             let step_box = Vec2::new(50.0, 50.0);
    //             for (step_transform, arm) in steps(escalator_transform, escalator_box, step_box)
    //                 .iter()
    //                 .take(1)
    //             {
    //                 let mut tmp = step_transform.clone();
    //                 tmp.translation.y -= 1.0;

    //                 spawn_step(
    //                     commands,
    //                     Handle::default(),
    //                     escalator,
    //                     tmp,
    //                     step_box,
    //                     arm.clone(),
    //                 );
    //             }
    //         },
    //         vec![(|steps: Query<(&Step, &Velocity)>| {
    //             for (_step, velocity) in steps.iter() {
    //                 assert_eq!(*velocity, Velocity(Some(Vec2::new(0.0, -2.0))));
    //             }
    //         })
    //         .system()],
    //     );
    // }

    // #[test]
    // fn knock_off() {
    //     helper(
    //         |commands, resources| {
    //             spawn_player(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(50.0, 50.0),
    //                 Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    //             );

    //             spawn_crate(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(50.0, 50.0),
    //                 Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
    //             );

    //             spawn_ground(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(50.0, 50.0),
    //                 Transform::from_translation(Vec3::new(50.0, 50.0, 0.0)),
    //             );

    //             spawn_ground(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(0.0, 50.0),
    //                 Transform::from_translation(Vec3::new(50.0, 50.0, 0.0)),
    //             );

    //             let mut input = Input::<KeyCode>::default();
    //             input.press(KeyCode::D);
    //             resources.insert(input)
    //         },
    //         vec![(|crates: Query<(&Crate, &Velocity)>| {
    //             for (_crate, velocity) in crates.iter() {
    //                 assert_eq!(*velocity, Velocity(Some(Vec2::new(0.0, -1.0))));
    //             }
    //         })
    //         .system()],
    //     );
    // }

    // #[test]
    // fn double_step() {
    //     helper(
    //         |commands, resources| {
    //             let escalator = spawn_escalator(
    //                 commands,
    //                 Handle::default(),
    //                 t(0.0, 0.0),
    //                 Vec2::new(200.0, 200.0),
    //             );

    //             spawn_step(
    //                 commands,
    //                 Handle::default(),
    //                 escalator,
    //                 t(-50.0, 50.0),
    //                 Vec2::new(50.0, 50.0),
    //                 Arm::D,
    //             );
    //             spawn_step(
    //                 commands,
    //                 Handle::default(),
    //                 escalator,
    //                 t(-75.0, 50.0),
    //                 Vec2::new(50.0, 50.0),
    //                 Arm::A,
    //             );

    //             spawn_ground(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(200.0, 100.0),
    //                 t(0.0, -150.0),
    //             );

    //             spawn_crate(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(50.0, 50.0),
    //                 t(-75.0, 100.0),
    //             );
    //         },
    //         vec![(|crates: Query<(&Crate, &Velocity)>| {
    //             for (_crate, velocity) in crates.iter() {
    //                 assert_eq!(*velocity, Velocity(Some(Vec2::new(-1.0, 1.0))));
    //             }
    //         })
    //         .system()],
    //     );
    // }

    // #[test]
    // fn walled_double_step() {
    //     helper(
    //         |commands, resources| {
    //             let escalator = spawn_escalator(
    //                 commands,
    //                 Handle::default(),
    //                 t(0.0, 0.0),
    //                 Vec2::new(200.0, 200.0),
    //             );

    //             spawn_step(
    //                 commands,
    //                 Handle::default(),
    //                 escalator,
    //                 t(-50.0, 50.0),
    //                 Vec2::new(50.0, 50.0),
    //                 Arm::D,
    //             );
    //             spawn_step(
    //                 commands,
    //                 Handle::default(),
    //                 escalator,
    //                 t(-75.0, 50.0),
    //                 Vec2::new(50.0, 50.0),
    //                 Arm::A,
    //             );

    //             spawn_ground(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(200.0, 100.0),
    //                 t(0.0, -150.0),
    //             );

    //             spawn_crate(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(50.0, 50.0),
    //                 t(-75.0, 100.0),
    //             );

    //             spawn_ground(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(50.0, 50.0),
    //                 t(-125.0, 75.0),
    //             );
    //         },
    //         vec![(|crates: Query<(&Crate, &Velocity)>| {
    //             for (_crate, velocity) in crates.iter() {
    //                 assert_eq!(*velocity, Velocity(Some(Vec2::new(0.0, 1.0))));
    //             }
    //         })
    //         .system()],
    //     );
    // }

    // #[test]
    // #[ignore]
    // fn pushing_walled_double_step() {
    //     helper(
    //         |commands, resources| {
    //             let escalator = spawn_escalator(
    //                 commands,
    //                 Handle::default(),
    //                 t(0.0, 0.0),
    //                 Vec2::new(200.0, 200.0),
    //             );

    //             spawn_step(
    //                 commands,
    //                 Handle::default(),
    //                 escalator,
    //                 t(-50.0, 50.0),
    //                 Vec2::new(50.0, 50.0),
    //                 Arm::D,
    //             );
    //             spawn_step(
    //                 commands,
    //                 Handle::default(),
    //                 escalator,
    //                 t(-75.0, 50.0),
    //                 Vec2::new(50.0, 50.0),
    //                 Arm::A,
    //             );

    //             spawn_ground(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(200.0, 100.0),
    //                 t(0.0, -150.0),
    //             );

    //             spawn_crate(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(50.0, 50.0),
    //                 t(-75.0, 100.0),
    //             );

    //             spawn_player(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(50.0, 50.0),
    //                 t(-125.0, 125.0),
    //             );
    //             spawn_ground(
    //                 commands,
    //                 Handle::default(),
    //                 Vec2::new(50.0, 50.0),
    //                 t(-125.0, 75.0),
    //             );

    //             let mut input = Input::<KeyCode>::default();
    //             input.press(KeyCode::D);
    //             resources.insert(input);
    //         },
    //         vec![(|crates: Query<(&Crate, &Velocity)>| {
    //             for (_crate, velocity) in crates.iter() {
    //                 assert_eq!(*velocity, Velocity(Some(Vec2::new(1.0, 1.0))));
    //             }
    //         })
    //         .system()],
    //     );
    // }
}
