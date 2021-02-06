use std::collections::{HashMap, HashSet};

use bevy::{diagnostic::Diagnostics, prelude::*};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_resource(AdjacencyGraph::default())
        .add_startup_system(setup2.system())
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
        // .add_system(initialize_velocity.system())
        // .add_system(propagate_velocity_horizontally.system())
        // .add_system(propagate_velocity_vertically.system())
        .add_system(update_position.system())
        .add_system(update_step_arm.system())
        // .add_system(x_collision_correction.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
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

enum Arm {
    A,
    B,
    C,
    D,
}

#[derive(Clone)]
struct Velocity(Option<Vec2>);

struct IntrinsicVelocity(Option<Vec2>);

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

fn setup2(
    commands: &mut Commands,

    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,

    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let walk_handle = asset_server.load("textures/base.png");
    let walk_atlas = TextureAtlas::from_grid(walk_handle, Vec2::new(200.0, 200.0), 1, 1);

    let walk_handle = texture_atlases.add(walk_atlas);

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            transform: Transform::from_translation(Vec3::new(100.0, 50.0, 1.0)),
            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
            ..Default::default()
        })
        .with(Player {})
        .with(BoundingBox(Vec2::new(50.0, 50.0)))
        .with(Velocity(None))
        .with(IntrinsicVelocity(None));

    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default());

    {
        let ground_box = Vec2::new(400.0, 50.0);
        commands
            .spawn(SpriteBundle {
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                sprite: Sprite::new(ground_box),
                ..Default::default()
            })
            .with(Ground {})
            .with(BoundingBox(ground_box))
            .with(Velocity(None));
    }

    {
        let ground_box = Vec2::new(400.0, 50.0);
        commands
            .spawn(SpriteBundle {
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(0.0, -150.0, 1.0)),
                sprite: Sprite::new(ground_box),
                ..Default::default()
            })
            .with(Ground {})
            .with(BoundingBox(ground_box))
            .with(Velocity(None));
    }
    {
        let ground_box = Vec2::new(400.0, 50.0);
        commands
            .spawn(SpriteBundle {
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(-50.0, -200.0, 1.0)),
                sprite: Sprite::new(ground_box),
                ..Default::default()
            })
            .with(Ground {})
            .with(BoundingBox(ground_box))
            .with(Velocity(None));
    }

    {
        let ground_box = Vec2::new(200.0, 200.0);
        commands
            .spawn(SpriteBundle {
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(-500.0, -75.0, 1.0)),
                sprite: Sprite::new(ground_box),
                ..Default::default()
            })
            .with(Ground {})
            .with(BoundingBox(ground_box))
            .with(Velocity(None));
    }

    {
        let ground_box = Vec2::new(50.0, 50.0);
        commands
            .spawn(SpriteBundle {
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(-500.0, 100.0, 1.0)),
                sprite: Sprite::new(ground_box),
                ..Default::default()
            })
            .with(Ground {})
            .with(BoundingBox(ground_box))
            .with(Velocity(None));
    }

    {
        let escalator_transform = Transform::from_translation(Vec3::new(-300.0, -75.0, 0.0));

        let escalator_box = BoundingBox(Vec2::new(200.0, 200.0));

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
                texture_atlas: walk_handle.clone_weak(),
                transform: escalator_transform,
                ..Default::default()
            })
            .with(Escalator {})
            .with(Velocity(None))
            .with(escalator_box.clone())
            .current_entity()
            .expect("Parent");

        let step_box = BoundingBox(Vec2::new(50.0, 50.0));
        for (step_transform, arm) in steps(&escalator_transform, &escalator_box, &step_box) {
            commands
                .spawn(SpriteBundle {
                    material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
                    transform: step_transform,
                    sprite: Sprite::new(Vec2::new(50.0, 50.0)),
                    ..Default::default()
                })
                .with(step_box.clone())
                .with(Step { arm, escalator })
                .with(Velocity(None))
                .with(IntrinsicVelocity(None));
        }
    }

    {
        let crate_box = Vec2::new(50.0, 50.0);

        commands
            .spawn(SpriteBundle {
                material: materials.add(Color::rgb(1.0, 0.5, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(0.0, 50.0, 1.0)),
                sprite: Sprite::new(crate_box),
                ..Default::default()
            })
            .with(Crate {})
            .with(BoundingBox(crate_box))
            .with(IntrinsicVelocity(None))
            .with(Velocity(None));
    }
}

fn setup(
    commands: &mut Commands,

    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,

    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let walk_handle = asset_server.load("textures/base.png");
    let walk_atlas = TextureAtlas::from_grid(walk_handle, Vec2::new(200.0, 200.0), 1, 1);

    let walk_handle = texture_atlases.add(walk_atlas);

    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default());

    // {
    //     let escalator_transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));

    //     let escalator_box = BoundingBox(Vec2::new(200.0, 200.0));

    //     let escalator = commands
    //         .spawn(SpriteSheetBundle {
    //             sprite: TextureAtlasSprite {
    //                 color: Color::rgba(1.0, 1.0, 1.0, 0.5),
    //                 ..TextureAtlasSprite::default()
    //             },

    //             visible: Visible {
    //                 is_visible: true,
    //                 is_transparent: true,
    //             },
    //             texture_atlas: walk_handle.clone_weak(),
    //             transform: escalator_transform,
    //             ..Default::default()
    //         })
    //         .with(Escalator {})
    //         .with(Velocity(None))
    //         .with(escalator_box.clone())
    //         .current_entity()
    //         .expect("Parent");

    //     let step_box = BoundingBox(Vec2::new(50.0, 50.0));
    //     for (step_transform, arm) in steps(&escalator_transform, &escalator_box, &step_box) {
    //         commands
    //             .spawn(SpriteBundle {
    //                 material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
    //                 transform: step_transform,
    //                 sprite: Sprite::new(Vec2::new(50.0, 50.0)),
    //                 ..Default::default()
    //             })
    //             .with(step_box.clone())
    //             .with(Step { arm, escalator })
    //             .with(Velocity(None))
    //             .with(IntrinsicVelocity(None));
    //     }
    // }

    {
        let ground_box = Vec2::new(400.0, 50.0);
        commands
            .spawn(SpriteBundle {
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(0.0, -200.0, 1.0)),
                sprite: Sprite::new(ground_box),
                ..Default::default()
            })
            .with(Ground {})
            .with(BoundingBox(ground_box))
            .with(Velocity(None));
    }

    {
        let ground_box = Vec2::new(400.0, 50.0);
        commands
            .spawn(SpriteBundle {
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(400.0, -150.0, 1.0)),
                sprite: Sprite::new(ground_box),
                ..Default::default()
            })
            .with(Ground {})
            .with(BoundingBox(ground_box))
            .with(Velocity(None));
    }

    {
        let ground_box = Vec2::new(400.0, 50.0);
        commands
            .spawn(SpriteBundle {
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(-400.0, -150.0, 1.0)),
                sprite: Sprite::new(ground_box),
                ..Default::default()
            })
            .with(Ground {})
            .with(BoundingBox(ground_box))
            .with(Velocity(None));
    }

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.5, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(100.0, 200.0, 1.0)),
            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
            ..Default::default()
        })
        .with(Crate {})
        .with(BoundingBox(Vec2::new(50.0, 50.0)))
        .with(Velocity(None));

    // commands
    //     .spawn(SpriteBundle {
    //         material: materials.add(Color::rgb(1.0, 0.5, 1.0).into()),
    //         transform: Transform::from_translation(Vec3::new(100.0, 250.0, 1.0)),
    //         sprite: Sprite::new(Vec2::new(50.0, 50.0)),
    //         ..Default::default()
    //     })
    //     .with(Crate {})
    //     .with(BoundingBox(Vec2::new(50.0, 50.0)))
    //     .with(Velocity(None));

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.5, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(00.0, 250.0, 1.0)),
            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
            ..Default::default()
        })
        .with(Crate {})
        .with(BoundingBox(Vec2::new(50.0, 50.0)))
        .with(Velocity(None))
        .with(IntrinsicVelocity(None));

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            transform: Transform::from_translation(Vec3::new(00.0, 150.0, 1.0)),
            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
            ..Default::default()
        })
        .with(Player {})
        .with(BoundingBox(Vec2::new(50.0, 50.0)))
        .with(Velocity(None))
        .with(IntrinsicVelocity(None));
}

fn steps(
    escalator_transform: &Transform,
    escalator_box: &BoundingBox,
    step: &BoundingBox,
) -> Vec<(Transform, Arm)> {
    let mut result = vec![];

    // A
    result.push((
        Transform::from_translation(Vec3::new(
            escalator_transform.translation.x - escalator_box.0.x / 2.0 + step.0.x / 2.0,
            escalator_transform.translation.y + escalator_box.0.y / 2.0 - step.0.y / 2.0,
            0.0,
        )),
        Arm::A,
    ));

    // B
    let n = (escalator_box.0.y / step.0.y) as i32;

    for index in 0..n - 2 {
        result.push((
            Transform::from_translation(Vec3::new(
                escalator_transform.translation.x - escalator_box.0.x / 2.0
                    + step.0.x / 2.0
                    + index as f32 * step.0.x,
                escalator_transform.translation.y + escalator_box.0.y / 2.0
                    - 3.0 * step.0.y / 2.0
                    - index as f32 * step.0.y,
                0.0,
            )),
            Arm::B,
        ))
    }

    // C
    result.push((
        Transform::from_translation(Vec3::new(
            escalator_transform.translation.x + escalator_box.0.x / 2.0 - 3.0 * step.0.y / 2.0,
            escalator_transform.translation.y - escalator_box.0.y / 2.0 + step.0.y / 2.0,
            0.0,
        )),
        Arm::C,
    ));

    // D
    for index in 0..n - 1 {
        result.push((
            Transform::from_translation(Vec3::new(
                escalator_transform.translation.x + escalator_box.0.x / 2.0
                    - step.0.x / 2.0
                    - (index as f32) * step.0.x,
                escalator_transform.translation.y
                    + -escalator_box.0.y / 2.0
                    + (step.0.y) / 2.0
                    + (index as f32) * step.0.y,
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

        // TODO: assign falling
        *velocity = IntrinsicVelocity(Some(Vec2::new(x_velocity, y_velocity)));
    }
}

fn step_intrinsic_velocity(mut query: Query<(&Step, &mut IntrinsicVelocity)>) {
    for (step, mut intrinsic_velocity) in query.iter_mut() {
        match step.arm {
            Arm::A => {
                *intrinsic_velocity = IntrinsicVelocity(Some(Vec2::new(0.0, -1.0)));
            }
            Arm::B => {
                *intrinsic_velocity = IntrinsicVelocity(Some(Vec2::new(1.0, -1.0)));
            }
            Arm::C => {
                *intrinsic_velocity = IntrinsicVelocity(Some(Vec2::new(1.0, 0.0)));
            }
            Arm::D => {
                *intrinsic_velocity = IntrinsicVelocity(Some(Vec2::new(-1.0, 1.0)));
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
                if (step_bottom - (escalator_top - 2.0 * step_box.0.y)).abs() < std::f32::EPSILON {
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

// fn initialize_velocity(
//     mut ungrounded: Query<(Entity, &mut Velocity), Without<Ground>>,
//     intrinsic_velocities: Query<&IntrinsicVelocity>,
// ) {
//     for (entity, mut velocity) in ungrounded.iter_mut() {
//         match intrinsic_velocities.get(entity) {
//             Ok(intrinsic_velocity) => {
//                 velocity.0 = intrinsic_velocity.0;
//             }
//             Err(_) => {
//                 velocity.0 = Vec2::zero();
//             }
//         }
//     }
// }

// you can push an escalator from the left, not the right
// pushing a step has to push the escalator
// pushable, non-pushable,

// do we just need to handle steps outside of this?

// fn propagate_velocity_horizontally(
//     left_query: Query<(Entity, &Transform, &BoundingBox), (Without<Escalator>)>,
//     right_query: Query<(Entity, &Transform, &BoundingBox), (Without<Step>)>,

//     ground_query: Query<&Ground>,
//     mut velocities: Query<&mut Velocity>,
// ) {
//     {
//         // might want to link steps / staircases?
//         // maintain L edges and R edges?

//         let mut left_edges: HashMap<Entity, HashSet<Entity>> = HashMap::new();
//         let mut left_bases: HashSet<Entity> = HashSet::new();
//         let mut left_nonbases: HashSet<Entity> = HashSet::new();

//         for (left_entity, left_transform, left_box) in left_query.iter() {
//             let mut beside_anything = false;

//             for (right_entity, right_transform, right_box) in right_query.iter() {
//                 if is_beside(left_transform, left_box, right_transform, right_box) {
//                     let current_lefts = left_edges.entry(left_entity).or_insert_with(HashSet::new);

//                     current_lefts.insert(right_entity);
//                     left_bases.insert(left_entity);
//                     left_nonbases.insert(right_entity);
//                     beside_anything = true;
//                 }
//             }

//             if !beside_anything {
//                 left_bases.insert(left_entity);
//             }
//         }

//         let left_roots = left_bases.difference(&left_nonbases);

//         let paths = build_paths(left_roots, left_edges);

//         // going left to right
//         for path in paths {
//             let mut max_velocity_so_far = 0.0;

//             let mut rightmost_ground: Option<usize> = None;

//             for (index, entity) in path.iter().enumerate() {
//                 // transfer positive x momentum only
//                 // don't want to swallow negative velocity (?)

//                 let mut node_velocity = velocities.get_mut(*entity).expect("velocity");

//                 if node_velocity.0.x >= 0.0 {
//                     node_velocity.0.x = node_velocity.0.x.max(max_velocity_so_far);
//                     max_velocity_so_far = node_velocity.0.x;
//                 }

//                 if ground_query.get(*entity).is_ok() {
//                     rightmost_ground = Some(index);
//                 }
//             }

//             // account for ground:
//             // everything left of last ground can only go left
//             if let Some(rightmost_ground) = rightmost_ground {
//                 for entity in path.iter().take(rightmost_ground + 1) {
//                     let mut node_velocity = velocities.get_mut(*entity).expect("velocity");
//                     node_velocity.0.x = node_velocity.0.x.min(0.0);
//                 }
//             }
//         }
//     }

//     {
//         // might want to link steps / staircases? depending on orientation?
//         // maintain L edges and R edges?

//         let mut right_edges: HashMap<Entity, HashSet<Entity>> = HashMap::new();
//         let mut right_bases: HashSet<Entity> = HashSet::new();
//         let mut right_nonbases: HashSet<Entity> = HashSet::new();

//         for (right_entity, right_transform, right_box) in right_query.iter() {
//             let mut beside_anything = false;

//             for (left_entity, left_transform, left_box) in left_query.iter() {
//                 if is_beside(left_transform, left_box, right_transform, right_box) {
//                     dbg!("beside!");
//                     let current_rights =
//                         right_edges.entry(right_entity).or_insert_with(HashSet::new);

//                     current_rights.insert(left_entity);
//                     right_bases.insert(right_entity);
//                     right_nonbases.insert(left_entity);
//                     beside_anything = true;
//                 }
//             }

//             if !beside_anything {
//                 right_bases.insert(right_entity);
//             }
//         }

//         let right_roots = right_bases.difference(&right_nonbases);

//         let paths = build_paths(right_roots, right_edges);

//         // going right to left
//         for path in paths {
//             let mut min_velocity_so_far = 0.0;
//             let mut leftmost_ground: Option<usize> = None;

//             for (index, entity) in path.iter().enumerate() {
//                 // transfer negative x momentum only
//                 // don't want to swallow positive velocity (?)
//                 let mut node_velocity = velocities.get_mut(*entity).expect("velocity");

//                 if node_velocity.0.x <= 0.0 {
//                     node_velocity.0.x = node_velocity.0.x.min(min_velocity_so_far);
//                     min_velocity_so_far = node_velocity.0.x;
//                 }

//                 if ground_query.get(*entity).is_ok() {
//                     leftmost_ground = Some(index);
//                 }
//             }

//             // account for ground:
//             // everything right of last ground can only go right
//             if let Some(leftmost_ground) = leftmost_ground {
//                 for entity in path.iter().take(leftmost_ground + 1) {
//                     let mut node_velocity = velocities.get_mut(*entity).expect("velocity");
//                     node_velocity.0.x = node_velocity.0.x.max(0.0);
//                 }
//             }
//         }
//     }
// }

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

// // TODO: maybe rewrite this using itertools instead of a QuerySet
// // See: https://docs.rs/itertools/0.10.0/itertools/trait.Itertools.html#method.permutations
// fn propagate_velocity_vertically(
//     atop_query: Query<(Entity, &Transform, &BoundingBox), Without<Step>>,
//     bases_query: Query<(Entity, &Transform, &BoundingBox), Without<Escalator>>,
//     steps: Query<(&Step, Entity)>,

//     grounds: Query<(&Ground, Entity)>,

//     mut velocities: Query<&mut Velocity>,
// ) {
//     let mut edges: HashMap<Entity, HashSet<Entity>> = HashMap::new();

//     let mut bases: HashSet<Entity> = HashSet::default();
//     let mut atops: HashSet<Entity> = HashSet::default();

//     for (step, step_entity) in steps.iter() {
//         let current_atops = edges.entry(step.escalator).or_insert_with(HashSet::new);
//         current_atops.insert(step_entity);
//         atops.insert(step_entity);
//         bases.insert(step.escalator);
//     }

//     for (atop_entity, atop_transform, atop_box) in atop_query.iter() {
//         let mut is_atop_anything = false;
//         for (below_entity, below_transform, below_box) in bases_query.iter() {
//             if is_atop(atop_transform, atop_box, below_transform, below_box) {
//                 is_atop_anything = true;
//                 let current_atops = edges.entry(below_entity).or_insert_with(HashSet::new);

//                 current_atops.insert(atop_entity);
//                 atops.insert(atop_entity);
//                 bases.insert(below_entity);
//             }
//         }
//         if !is_atop_anything {
//             bases.insert(atop_entity);
//         }
//     }

//     let roots = bases.difference(&atops);

//     // can't just do simple min cuz want -2 :(
//     let mut already_visited: HashMap<Entity, Vec2> = HashMap::new();

//     for path in build_paths(roots, edges) {
//         let mut cumulative_velocity = Vec2::new(0.0, -1.0);

//         for entity in path.iter() {
//             if grounds.get(*entity).is_ok() {
//                 cumulative_velocity = Vec2::zero();
//             };

//             let node_velocity = velocities.get_mut(*entity).expect("velocity query");
//             let proposed_velocity = node_velocity.0 + cumulative_velocity;

//             let canonical_velocity = already_visited.entry(*entity).or_insert(proposed_velocity);

//             if canonical_velocity.y < proposed_velocity.y {
//                 *canonical_velocity = proposed_velocity;
//             }

//             cumulative_velocity = proposed_velocity;
//         }
//     }

//     for (entity, velocity) in already_visited.iter() {
//         if let Err(e) = velocities.set(*entity, Velocity(*velocity)) {
//             panic!("Error setting velocity: {:?}", e);
//         }
//     }
// }

// generates all complete paths
fn build_paths<'a>(
    roots: impl Iterator<Item = &'a Entity>,
    edges: HashMap<Entity, HashSet<Entity>>,
) -> Vec<Vec<Entity>> {
    let mut result = vec![];

    for root in roots {
        result.extend(path_helper(*root, &edges));
    }

    result
}

fn path_helper(current: Entity, edges: &HashMap<Entity, HashSet<Entity>>) -> Vec<Vec<Entity>> {
    // base case, no edges

    match edges.get(&current) {
        Some(children) => {
            let mut result = vec![];

            for child in children {
                for mut child_path in path_helper(*child, edges) {
                    child_path.insert(0, current);
                    result.push(child_path);
                }
            }

            result
        }
        None => vec![vec![current]],
    }
}

fn x_collision_correction(
    mut crates: Query<(&Crate, &mut Transform, &BoundingBox)>,
    steps: Query<(&Step, &Transform, &BoundingBox)>,
) {
    for (_crate, mut crate_transform, crate_box) in crates.iter_mut() {
        let crate_top = crate_transform.translation.y + crate_box.0.y / 2.0;
        let crate_bottom = crate_transform.translation.y - crate_box.0.y / 2.0;
        let crate_left = crate_transform.translation.x - crate_box.0.x / 2.0;
        let crate_right = crate_transform.translation.x + crate_box.0.x / 2.0;

        for (_step, step_transform, step_box) in steps.iter() {
            let step_top = step_transform.translation.y + step_box.0.y / 2.0;
            let step_bottom = step_transform.translation.y - step_box.0.y / 2.0;
            let step_left = step_transform.translation.x - step_box.0.x / 2.0;
            let step_right = step_transform.translation.x + step_box.0.x / 2.0;

            if (step_bottom <= crate_bottom && step_top > crate_bottom)
                || (crate_bottom <= step_bottom && crate_top > step_bottom)
            {
                if step_left <= crate_left && step_right > crate_left {
                    let delta = step_right - crate_left;

                    crate_transform.translation.x += delta;
                }

                if crate_left <= step_left && crate_right > step_left {
                    let delta = crate_right - step_left;

                    crate_transform.translation.x -= delta;
                }
            }
        }
    }
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
    right_query: Query<(Entity, &Transform, &BoundingBox), (Without<Step>)>,
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
                    *intrinsic_velocity = IntrinsicVelocity(Some(Vec2::new(0.0, -1.0)));
                }
            }
            None => {
                *intrinsic_velocity = IntrinsicVelocity(Some(Vec2::new(0.0, -1.0)));
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
) {
    // order intrinsic velocities by y top

    let mut intrinsic_velocity_sources = vec![];

    for (entity, transform, bounding_box, intrinsic_velocity) in order_query.iter() {
        if let Some(intrinsic_velocity) = intrinsic_velocity.0 {
            let top = transform.translation.y + bounding_box.0.y / 2.0;

            intrinsic_velocity_sources.push((entity, top, intrinsic_velocity));
        }
    }

    intrinsic_velocity_sources.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("sort velocity"));

    for (entity, _top, intrinsic_velocity) in intrinsic_velocity_sources {
        propagate_velocity(
            entity,
            intrinsic_velocity,
            &*adjacency_graph,
            &grounds,
            &mut velocities,
        );
    }
}

// maybe want a "dry run" option

// returns true if propagation down this direction hit a Ground
// which velocities are additive, and which are not?
// if y velocity is increasing, ditch old stuff (?)
// if y velocity is geq, add (?)

fn propagate_velocity(
    entity: Entity,
    propagation_velocity: Vec2,
    adjacency_graph: &AdjacencyGraph,
    grounds: &Query<&Ground>,
    mut velocities: &mut Query<&mut Velocity>,
) -> bool {
    if grounds.get(entity).is_ok() {
        return true;
    }

    // handle x first
    if propagation_velocity.x < 0.0 {
        let mut any_ground = false;
        let x_velocity = Vec2::new(propagation_velocity.x, 0.0);

        if let Some(left_entities) = adjacency_graph.lefts.get(&entity) {
            for left_entity in left_entities {
                any_ground = any_ground
                    | propagate_velocity(
                        *left_entity,
                        x_velocity,
                        adjacency_graph,
                        grounds,
                        velocities,
                    );
            }

            if any_ground {
                for left_entity in left_entities {
                    propagate_velocity(
                        *left_entity,
                        Vec2::zero(),
                        adjacency_graph,
                        grounds,
                        velocities,
                    );
                }
            }
        }

        if !any_ground {
            let mut node_velocity = velocities.get_mut(entity).expect("velocity");

            match node_velocity.0 {
                Some(prior_velocity) => {
                    let new_velocity =
                        Vec2::new(prior_velocity.x.min(x_velocity.x), prior_velocity.y);
                    node_velocity.0 = Some(new_velocity);
                }
                None => {
                    node_velocity.0 = Some(x_velocity);
                }
            }
        }
    }
    if propagation_velocity.x > 0.0 {
        let mut any_ground = false;
        let x_velocity = Vec2::new(propagation_velocity.x, 0.0);

        if let Some(right_entities) = adjacency_graph.rights.get(&entity) {
            for right_entity in right_entities {
                any_ground = any_ground
                    | propagate_velocity(
                        *right_entity,
                        x_velocity,
                        adjacency_graph,
                        grounds,
                        velocities,
                    )
            }

            if any_ground {
                for right_entity in right_entities {
                    propagate_velocity(
                        *right_entity,
                        Vec2::zero(),
                        adjacency_graph,
                        grounds,
                        velocities,
                    );
                }
            }
        }

        if !any_ground {
            let mut node_velocity = velocities.get_mut(entity).expect("velocity");

            match node_velocity.0 {
                Some(prior_velocity) => {
                    // maybe x should be additive here?
                    let new_velocity =
                        Vec2::new(prior_velocity.x.max(x_velocity.x), prior_velocity.y);
                    node_velocity.0 = Some(new_velocity);
                }
                None => {
                    node_velocity.0 = Some(x_velocity);
                }
            }
        }
    }

    // handle y
    {
        let mut any_ground = false;

        if propagation_velocity.y > 0.0 {
            if let Some(tops) = adjacency_graph.tops.get(&entity) {
                for top_entity in tops {
                    any_ground = any_ground
                        | propagate_velocity(
                            *top_entity,
                            propagation_velocity,
                            adjacency_graph,
                            grounds,
                            velocities,
                        );
                }

                if any_ground {
                    for top_entity in tops {
                        propagate_velocity(
                            *top_entity,
                            Vec2::zero(),
                            adjacency_graph,
                            grounds,
                            velocities,
                        );
                    }
                }
            }
        }

        let mut node_velocity = velocities.get_mut(entity).expect("velocity");

        if any_ground {
            node_velocity.0.unwrap().y = 0.0;
        } else {
            if propagation_velocity.y == 0.0 {
                return false;
            }

            match node_velocity.0 {
                Some(mut old_velocity) => {
                    old_velocity.y = propagation_velocity.y;

                    *node_velocity = Velocity(Some(old_velocity));
                }
                None => node_velocity.0 = Some(Vec2::new(0.0, propagation_velocity.y)),
            }
        }
    }

    false
}
