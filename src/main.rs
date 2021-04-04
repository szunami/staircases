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
        // systems that don't edit velocity
        .add_system(update_step_track.system())
        // first pass at setting velocities
        .add_system(reset_velocity.system())
        .add_system(step_velocity.system())
        .add_system(player_velocity.system())
        .add_system(falling_velocity.system())
        .add_system(normal_force.system())
        .add_system(friction.system())
        .add_system(ladder.system())
        // integrate
        .add_system(update_position.system())
        .add_system(
            (|q: Query<(&Crate, &Transform, &Velocity)>| {
                // dbg!("first pass");
                for (_crate, xform, velocity) in q.iter() {
                    // dbg!(xform.translation);
                }
            })
            .system(),
        )
        // second pass at setting velocities; impulses to avoid collisions
        // TODO: carry
        .add_system(reset_velocity.system())
        .add_system(process_collisions.system())
        .add_system(update_position.system())
        // .add_system(reset_velocity.system())
        // .add_system(process_collisions.system())
        // .add_system(update_position.system())
        // .add_system(reset_velocity.system())
        // .add_system(process_collisions.system())
        // .add_system(update_position.system())
        .add_system(lines.system())
        .add_system(
            (|q: Query<(&Player, &Transform, &Velocity)>,
              r: Query<(&Crate, &Transform, &Velocity)>| {
                // dbg!("player");
                for (_player, xform, velocity) in q.iter() {
                    // dbg!(xform.translation);
                }
                // dbg!("crate");
                for (_crate, xform, velocity) in r.iter() {
                    // dbg!(xform.translation);
                }
            })
            .system(),
        )
        .run();
}

fn falling_velocity(mut q: Query<&mut Velocity>) {
    for mut velocity in q.iter_mut() {
        velocity.0.y -= 1.0;
    }
}

fn normal_force(
    time: Res<Time>,
    q: Query<(Entity, &Transform, &ConvexPolygon), Without<Ladder>>,

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

                if let Ok(_step_b) = steps.get(entity_b) {
                    continue;
                }
            }

            if let Ok(step) = steps.get(entity_b) {
                if step.escalator == entity_a {
                    continue;
                }
            }

            if let Some(contact) = collision(poly_a, &xform_a, poly_b, &xform_b) {
                // dbg!(contact.clone());

                // HACK: collisions shouldn't push down(?)

                if contact.normal1.y < 0. {
                    // apply normal force to a

                    if let Ok(mut velocity_a) = velocities.get_mut(entity_a) {
                        velocity_a.0.y += 1.0;
                    }
                }

                if contact.normal2.y < 0.0 {
                    // apply normal force to b

                    if let Ok(mut velocity_b) = velocities.get_mut(entity_b) {
                        velocity_b.0.y += 1.0;
                    }
                }
            }
        }
    }
}

/*
Friction is applied between two bodies in contact.
It is perpendicular to the normal of the comment
and resists motion of the top entity relative to the bottom entity.
*/
fn friction(
    time: Res<Time>,
    q: Query<(Entity, &Transform, &ConvexPolygon), Without<Ladder>>,

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

                if let Ok(_step_b) = steps.get(entity_b) {
                    continue;
                }
            }

            if let Ok(step) = steps.get(entity_b) {
                if step.escalator == entity_a {
                    continue;
                }
            }

            if let Some(contact) = collision(poly_a, &xform_a, poly_b, &xform_b) {
                // friction should be
                // proportional to velocity
                // orthogonal to normal

                // friction from b to a:

                let FRICTION_COEFFICIENT: f32 = 1.0;

                if contact.normal2.y > 0. {
                    if let Ok(velocity_b) = velocities.get_mut(entity_b) {
                        let velocity_b = velocity_b.clone();

                        if let Ok(mut velocity_a) = velocities.get_mut(entity_a) {
                            let friction = FRICTION_COEFFICIENT
                                * velocity_b.0
                                * contact.normal1.perp().normalize();

                            // project b's velocity onto
                            velocity_a.0 += friction;
                        }
                    }
                }

                if contact.normal1.y > 0. {
                    if let Ok(velocity_a) = velocities.get_mut(entity_a) {
                        let velocity_a = velocity_a.clone();

                        if let Ok(mut velocity_b) = velocities.get_mut(entity_b) {
                            // project b's velocity onto
                            let friction = FRICTION_COEFFICIENT
                                * velocity_a.0
                                * contact.normal2.perp().normalize();

                            velocity_b.0 += friction;
                        }
                    }
                }
            }
        }
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
    escalator: Entity,
    length: f32,
}

#[derive(Debug)]
struct Track {
    position: f32,
    length: f32,
}

#[derive(Clone, PartialEq, Debug)]
struct Velocity(Vec2);

struct Ground;

#[derive(PartialEq, Eq, Hash)]
struct Crate;

struct Player;

struct Ladder;

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
            Vec2::new(50., 50.),
            t(0., -25.),
        );

        let escalator = spawn_escalator(commands, escalator_handle.clone_weak(), t(0., 100.), 200.);

        for (step_transform, track_position, track_length) in steps(t(0., 100.), 200., 50.) {
            spawn_step(
                commands,
                step_handle.clone_weak(),
                escalator,
                step_transform,
                50.0,
                track_position,
                track_length,
            );
        }

        spawn_player(
            commands,
            player_handle.clone_weak(),
            Vec2::new(50.0, 100.0),
            t(0.0, 300.0),
        );

        spawn_ground(
            commands,
            ground_handle.clone_weak(),
            Vec2::new(50.0, 50.0),
            t(-125., 175.),
        );

        // spawn_crate(
        //     commands,
        //     crate_handle.clone_weak(),
        //     Vec2::new(50.0, 50.0),
        //     t(0.0, 50.0),
        // );

        // spawn_crate(
        //     commands,
        //     crate_handle.clone_weak(),
        //     Vec2::new(50.0, 50.0),
        //     t(-200.0, 0.0),
        // );

        // spawn_ground(
        //     commands,
        //     ground_handle.clone_weak(),
        //     Vec2::new(50.0, 50.0),
        //     t(-200.0, -50.0),
        // );

        // spawn_ground(
        //     commands,
        //     ground_handle.clone_weak(),
        //     Vec2::new(50.0, 50.0),
        //     t(-250.0, 0.0),
        // );
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
            transform,
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
fn spawn_ladder(
    commands: &mut Commands,
    material: Handle<ColorMaterial>,
    transform: Transform,
    size: Vec2,
) {
    commands
        .spawn(SpriteBundle {
            material,
            transform,
            sprite: Sprite::new(size),
            ..Default::default()
        })
        .with(Ladder)
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
fn spawn_step(
    commands: &mut Commands,
    material: Handle<ColorMaterial>,
    escalator: Entity,
    transform: Transform,
    length: f32,
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
        .with(Step { escalator, length })
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
            ])
            .expect("poly"),
        )
        .current_entity()
        .expect("Spawned crate")
}

#[allow(dead_code)]
fn steps(
    escalator_transform: Transform,
    escalator_length: f32,
    step_length: f32,
) -> Vec<(Transform, f32, f32)> {
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
            track_position,
            track_length,
        ));
        track_position += step_length;
    }
    result
}

fn player_velocity(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &mut Velocity)>,
) {
    for (_player, mut velocity) in query.iter_mut() {
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

fn step_velocity(
    mut step_query: Query<(&Step, &Track, &Transform, &mut Velocity)>,
    escalator_query: Query<(&Escalator, &Transform)>,
) {
    for (step, track, step_transform, mut velocity) in step_query.iter_mut() {
        let (escalator, escalator_transform) = escalator_query
            .get(step.escalator)
            .expect("Step escalator lookup");

        let s = step.length;
        let n = escalator.length / s;

        let t1 = s;
        let t2 = s + (n - 1.) * s;
        let t3 = 2. * s + (n - 1.) * s;

        let t = track.position;

        let target = escalator_transform.translation.truncate() + {
            if t < t1 {
                Vec2::new(-(n - 3.0) * s / 2.0, (n - 1.0) * s / 2.0) + Vec2::new(-t, 0.0)
            } else if t < t2 {
                Vec2::new(-(n - 1.0) * s / 2.0, (n - 1.0) * s / 2.0) + Vec2::new(t - t1, -(t - t1))
            } else if t < t3 {
                Vec2::new((n - 1.) * s / 2., -(n - 1.) * s / 2.) + Vec2::new(t - t2, 0.0)
            } else {
                Vec2::new((n + 1.) * s / 2., -(n - 1.) * s / 2.) + Vec2::new(-(t - t3), t - t3)
            }
        };

        *velocity = Velocity(target - step_transform.translation.truncate());
    }
}

fn process_collisions(
    time: Res<Time>,
    q: Query<(Entity, &Transform, &ConvexPolygon), Without<Ladder>>,

    mut velocities: Query<&mut Velocity>,

    steps: Query<&Step>,
) {
    // HACK: this will get multiplied by delta, so we divide by it first
    let delta = BASE_SPEED_FACTOR * time.delta_seconds();

    if delta == 0.0 {
        return;
    }

    for (entity_a, xform_a, poly_a) in q.iter() {
        for (entity_b, xform_b, poly_b) in q.iter() {
            if entity_a >= entity_b {
                continue;
            }

            if let Ok(step_a) = steps.get(entity_a) {
                if step_a.escalator == entity_b {
                    continue;
                }

                if let Ok(_step_b) = steps.get(entity_b) {
                    continue;
                }
            }

            if let Ok(step) = steps.get(entity_b) {
                if step.escalator == entity_a {
                    continue;
                }
            }

            if let Some(contact) = collision(poly_a, &xform_a, poly_b, &xform_b) {
                // dbg!(contact.clone());

                // HACK: collisions shouldn't push down(?)

                if velocities.get_mut(entity_a).is_ok() && velocities.get_mut(entity_b).is_ok() {
                    {
                        let mut collision_correction = contact.normal1 * contact.dist;
                        collision_correction.y = collision_correction.y.max(0.0);

                        let mut velocity_a = velocities.get_mut(entity_a).unwrap();
                        *velocity_a = Velocity(velocity_a.0 + collision_correction / delta);
                    }

                    {
                        let mut collision_correction = contact.normal2 * contact.dist;
                        collision_correction.y = collision_correction.y.max(0.0);

                        let mut velocity_b = velocities.get_mut(entity_b).unwrap();
                        *velocity_b = Velocity(velocity_b.0 + collision_correction / delta);
                    }
                } else if let Ok(mut w) = velocities.get_mut(entity_a) {
                    let collision_correction = contact.normal1 * contact.dist;
                    *w = Velocity(w.0 + collision_correction / delta);
                } else if let Ok(mut r) = velocities.get_mut(entity_b) {
                    let collision_correction: Vec2 = contact.normal2 * contact.dist;
                    *r = Velocity(r.0 + collision_correction / delta);
                } else {
                }
            }
        }
    }
}

fn update_position(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    let delta = BASE_SPEED_FACTOR * time.delta_seconds();
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += delta * velocity.0.x;
        transform.translation.y += delta * velocity.0.y;
    }
}

fn update_step_track(time: Res<Time>, mut steps: Query<(&Step, &mut Track)>) {
    let delta = BASE_SPEED_FACTOR * time.delta_seconds();

    for (_step, mut track) in steps.iter_mut() {
        track.position = (track.position + delta) % track.length;
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

#[derive(Debug, Clone)]
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

    let epsilon = 0.0001;
    query::contact(&i1, poly1, &i2, poly2, 0.1)
        .map(|contact| {
            contact.map(|contact| {
                if contact.dist >= epsilon {
                    return None;
                }

                Some(BevyCollision {
                    normal1: Vec2::new(contact.normal1.x, contact.normal1.y),
                    normal2: Vec2::new(contact.normal2.x, contact.normal2.y),
                    dist: contact.dist,
                })
            })
        })
        .ok()
        .flatten()
        .flatten()
}

const LADDER_TOLERANCE: f32 = 2.0;

fn ladder(
    keys: Res<Input<KeyCode>>,

    mut players: Query<(&Player, &Transform, &ConvexPolygon, &mut Velocity)>,
    ladders: Query<(&Ladder, &Transform, &ConvexPolygon)>,
) {
    for (_player, player_xform, player_poly, mut player_velocity) in players.iter_mut() {
        for (_ladder, ladder_xform, ladder_poly) in ladders.iter() {
            if let Some(_collision) =
                collision(player_poly, player_xform, ladder_poly, ladder_xform)
            {
                if (player_xform.translation.x - ladder_xform.translation.x).abs()
                    < LADDER_TOLERANCE
                    && keys.pressed(KeyCode::W)
                {
                    player_velocity.0.x = ladder_xform.translation.x - player_xform.translation.x;
                    player_velocity.0.y = 1.0;
                }
            }
        }
    }
}
