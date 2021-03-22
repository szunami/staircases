use bevy::{diagnostic::Diagnostics, prelude::*};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use nalgebra::{Isometry2, Point2, Vector2};
use parry2d::{query, shape::ConvexPolygon};

const BASE_SPEED_FACTOR: f32 = 70.0;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_startup_system(setup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        // .add_system(framerate.system())
        .add_system(update_step_arm.system())
        .add_system(update_step_track.system())
        // reset IV
        .add_system(reset_velocity.system())
        // assign step IV
        .add_system(step_intrinsic_velocity.system())
        // assign player IV
        .add_system(player_intrinsic_velocity.system())
        // assign falling IV
        .add_system(falling.system())
        // propagation
        // .add_system(velocity_propagation.system())
        // reset velocity
        .add_system(process_collisions.system())
        // for each IV, in order of ascending y, propagate
        .add_system(update_position.system())
        .add_system(lines.system())
        .run();
}

fn falling(mut q: Query<&mut Velocity>) {
    for mut velocity in q.iter_mut() {
        velocity.0.y -= 1.0;
    }
}

#[allow(dead_code)]
fn framerate(diagnostics: Res<Diagnostics>) {
    if let Some(fps) = diagnostics.get(bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
        dbg!(fps.average());
    }
}

struct Escalator {
    length: f32,
}

struct Step {
    arm: Arm,
    escalator: Entity,
    length: f32,
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
struct Velocity(Vec2);

struct Ground;

#[derive(PartialEq, Eq, Hash)]
struct Crate;

struct Player;

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

        spawn_crate(
            commands,
            crate_handle.clone_weak(),
            Vec2::new(50.0, 50.0),
            t(100.0, 400.0),
        );

        spawn_player(
            commands,
            player_handle.clone_weak(),
            Vec2::new(50.0, 100.0),
            t(0.0, 300.0),
        );

        let escalator_transform = t(50.0, 50.0);
        let escalator_length = 200.0;
        let escalator = spawn_escalator(
            commands,
            escalator_handle.clone_weak(),
            escalator_transform,
            escalator_length,
        );

        let step_length = 50.0;
        for (step_transform, arm, track_position, track_length) in
            steps(escalator_transform, escalator_length, step_length).iter()
        {
            spawn_step(
                commands,
                step_handle.clone_weak(),
                escalator,
                *step_transform,
                step_length,
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
    length: f32,
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
        .with(Escalator { length })
        .with(Velocity(Vec2::zero()))
        .with(
            ConvexPolygon::from_convex_hull(&[
                Point2::new(-length / 2.0, length / 2.0 - 10.0),
                Point2::new(length / 2.0, -length / 2.0),
                Point2::new(-length / 2.0, -length / 2.0),
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
    length: f32,
    arm: Arm,
    track_position: f32,
    track_length: f32,
) -> Entity {
    commands
        .spawn(SpriteBundle {
            material,
            transform,
            sprite: Sprite::new(Vec2::splat(length)),
            ..Default::default()
        })
        .with(Step {
            arm,
            escalator,
            length,
        })
        .with(Velocity(Vec2::zero()))
        .with(Track {
            length: track_length,
            position: track_position,
        })
        .with(
            ConvexPolygon::from_convex_hull(&[
                Point2::new(-length / 2.0, length / 2.0),
                Point2::new(length / 2.0, length / 2.0),
                Point2::new(length / 2.0, -length / 2.0),
                Point2::new(-length / 2.0, -length / 2.0),
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
        .with(Velocity(Vec2::zero()))
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
        .with(Velocity(Vec2::zero()))
        .with(
    ConvexPolygon::from_convex_hull(&[
                Point2::new(-size.x / 2.0, size.y / 2.0),
                Point2::new(size.x / 2.0, size.y / 2.0),
                Point2::new(size.x / 2.0, -size.y / 2.0),
                Point2::new(-size.x / 2.0, -size.y / 2.0),
            ]).expect("poly"))
        .current_entity()
        .expect("Spawned crate")
}

#[allow(dead_code)]
fn steps(
    escalator_transform: Transform,
    escalator_length: f32,
    step_length: f32,
) -> Vec<(Transform, Arm, f32, f32)> {
    let mut result = vec![];
    let n = (escalator_length / step_length) as i32;

    let track_length = (2.0 * (n as f32 - 1.0) + 2.0) * step_length;
    let mut track_position = 0.0;

    // A
    result.push((
        Transform::from_translation(Vec3::new(
            escalator_transform.translation.x - escalator_length / 2.0 + step_length / 2.0,
            escalator_transform.translation.y + escalator_length / 2.0 - step_length / 2.0,
            0.0,
        )),
        Arm::A,
        track_position,
        track_length,
    ));

    track_position += step_length;

    // B

    for index in 0..n - 2 {
        result.push((
            Transform::from_translation(Vec3::new(
                escalator_transform.translation.x - escalator_length / 2.0
                    + step_length / 2.0
                    + index as f32 * step_length,
                escalator_transform.translation.y + escalator_length / 2.0
                    - 3.0 * step_length / 2.0
                    - index as f32 * step_length,
                0.0,
            )),
            Arm::B,
            track_position,
            track_length,
        ));
        track_position += step_length;
    }

    // C
    result.push((
        Transform::from_translation(Vec3::new(
            escalator_transform.translation.x + escalator_length / 2.0 - 3.0 * step_length / 2.0,
            escalator_transform.translation.y - escalator_length / 2.0 + step_length / 2.0,
            0.0,
        )),
        Arm::C,
        track_position,
        track_length,
    ));
    track_position += step_length;

    // D
    for index in 0..n {
        result.push((
            Transform::from_translation(Vec3::new(
                escalator_transform.translation.x + escalator_length / 2.0
                    - step_length / 2.0
                    - (index as f32) * step_length,
                escalator_transform.translation.y
                    + -escalator_length / 2.0
                    + step_length / 2.0
                    + (index as f32) * step_length,
                0.0,
            )),
            Arm::D,
            track_position,
            track_length,
        ));
        track_position += step_length;
    }
    result
}

fn player_intrinsic_velocity(
    keyboard_input: Res<Input<KeyCode>>,
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

        *velocity = Velocity(Vec2::new(x_velocity, 0.0));
    }
}

fn step_intrinsic_velocity(
    mut step_query: Query<(&Step, &Track, &Transform, &mut Velocity)>,
    escalator_query: Query<(&Escalator, &Transform)>,
) {
    for (step, track, step_transform, mut velocity) in step_query.iter_mut() {
        let (escalator, escalator_transform) = escalator_query
            .get(step.escalator)
            .expect("Step escalator lookup");

        let s = step.length;
        let N = escalator.length / s;

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

        *velocity = Velocity(target - step_transform.translation.truncate());
    }
}

fn process_collisions(
    q: Query<(Entity, &Transform, &ConvexPolygon)>,

    mut velocities: Query<&mut Velocity>,

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
                if velocities.get_mut(entity_a).is_ok() && velocities.get_mut(entity_b).is_ok() {
                    {
                        let velocity_b = velocities.get_mut(entity_b).unwrap().clone();
                        let mut velocity_a = velocities.get_mut(entity_a).unwrap();
                        if contact.normal1.y < 0.0 {
                            let carry_v = Vec2::new(velocity_b.0.x, 0.0);
                            // b carrying a
                            *velocity_a = Velocity(velocity_a.0 + carry_v);
                        }
                        *velocity_a = Velocity(velocity_a.0 + contact.normal1 * contact.dist / 2.0);
                    }

                    {
                        let velocity_a = velocities.get_mut(entity_a).unwrap().clone();
                        let mut velocity_b = velocities.get_mut(entity_b).unwrap();
                        if contact.normal2.y < 0.0 {
                            let carry_v = Vec2::new(velocity_a.0.x, 0.0);
                            // a carrying b
                            *velocity_b = Velocity(velocity_b.0 + carry_v);
                        }
                        *velocity_b = Velocity(velocity_b.0 + contact.normal2 * contact.dist / 2.0);
                    }
                } else if let Ok(mut w) = velocities.get_mut(entity_a) {
                    *w = Velocity(w.0 + contact.normal1 * contact.dist);
                } else if let Ok(mut r) = velocities.get_mut(entity_b) {
                    *r = Velocity(r.0 + contact.normal2 * contact.dist / 2.0);
                } else {
                }
            }
        }
    }
}

fn update_position(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    let delta_seconds = BASE_SPEED_FACTOR * time.delta().as_secs_f32();

    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += delta_seconds * velocity.0.x;
        transform.translation.y += delta_seconds * velocity.0.y;
    }
}

fn update_step_track(time: Res<Time>, mut steps: Query<(&Step, &mut Track)>) {
    let delta = BASE_SPEED_FACTOR * time.delta_seconds();

    for (_step, mut track) in steps.iter_mut() {
        track.position = (track.position + delta) % track.length;
        // dbg!(track);
    }
}

fn update_step_arm(
    commands: &mut Commands,
    mut steps: Query<(Entity, &mut Step, &mut Track, &Transform)>,
    escalators: Query<(&Escalator, &Transform)>,
) {
    for (step_entity, mut step, track, step_transform) in steps.iter_mut() {
        let (escalator, escalator_transform) =
            escalators.get(step.escalator).expect("fetch escalator");

        let step_top = step_transform.translation.y + step.length / 2.0;
        let step_bottom = step_transform.translation.y - step.length / 2.0;
        let step_right = step_transform.translation.x + step.length / 2.0;

        let escalator_top = escalator_transform.translation.y + escalator.length / 2.0;
        let escalator_bottom = escalator_transform.translation.y - escalator.length / 2.0;
        let escalator_right = escalator_transform.translation.x + escalator.length / 2.0;

        match step.arm {
            Arm::A => {
                if (step_bottom - (escalator_top - 2.0 * step.length)) < std::f32::EPSILON {
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

fn reset_velocity(mut query: Query<&mut Velocity>) {
    for mut velocity in query.iter_mut() {
        *velocity = Velocity(Vec2::zero());
    }
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

// #[cfg(test)]
// mod tests {

//     use bevy::ecs::{FuncSystem, Stage};

//     use super::*;

//     fn helper<F>(commands_init: F, assertions: Vec<FuncSystem<()>>)
//     where
//         F: FnOnce(&mut Commands, &mut Resources) -> (),
//     {
//         let mut world = World::default();
//         let mut resources = Resources::default();

//         resources.insert(Input::<KeyCode>::default());

//         let mut commands = Commands::default();

//         commands.set_entity_reserver(world.get_entity_reserver());

//         commands_init(&mut commands, &mut resources);
//         commands.apply(&mut world, &mut resources);

//         let mut stage = SystemStage::serial();

//         stage
//             // .add_system(bevy::input::system::exit_on_esc_system.system())
//             // .add_system(framerate.system())
//             // reset IV
//             // build edge graph
//             // assign step IV
//             .add_system(step_intrinsic_velocity.system())
//             // assign player IV
//             .add_system(player_intrinsic_velocity.system())
//             // assign falling IV
//             // propagation
//             .add_system(reset_velocity.system())
//             // reset velocity
//             // for each IV, in order of ascending y, propagate
//             .add_system(update_position.system())
//             .add_system(update_step_arm.system());

//         for system in assertions {
//             stage.add_system(system);
//         }

//         stage.initialize(&mut world, &mut resources);

//         stage.run_once(&mut world, &mut resources)
//     }
// }
