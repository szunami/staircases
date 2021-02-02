use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(step_intrinsic_velocity.system())
        .add_system(ground_velocity.system())
        .add_system(propagate_velocity.system())
        .add_system(update_position.system())
        .add_system(update_step_arm.system())
        .add_system(x_collision_correction.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
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
struct Velocity(Vec2);

struct IntrinsicVelocity(Vec2);

struct Ground;

#[derive(PartialEq, Eq, Hash)]
struct Crate;

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
    //     let escalator_transform = Transform::from_translation(Vec3::new(100.0, 0.0, 0.0));

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
    //         .with(Velocity(Vec2::zero()))
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
    //             .with(Velocity(Vec2::zero()))
    //             .with(IntrinsicVelocity(Vec2::zero()));
    //     }
    // }

    {
        let escalator_transform = Transform::from_translation(Vec3::new(100.0, 500.0, 0.0));

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
                texture_atlas: walk_handle,
                transform: escalator_transform,
                ..Default::default()
            })
            .with(Escalator {})
            .with(Velocity(Vec2::zero()))
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
                .with(Velocity(Vec2::zero()))
                .with(IntrinsicVelocity(Vec2::zero()));
        }
    }

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, -200.0, 1.0)),
            sprite: Sprite::new(Vec2::new(200.0, 50.0)),
            ..Default::default()
        })
        .with(Ground {})
        .with(BoundingBox(Vec2::new(200.0, 50.0)))
        .with(Velocity(Vec2::zero()));

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.5, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(100.0, 200.0, 1.0)),
            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
            ..Default::default()
        })
        .with(Crate {})
        .with(BoundingBox(Vec2::new(50.0, 50.0)))
        .with(Velocity(Vec2::zero()));

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.5, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(100.0, 250.0, 1.0)),
            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
            ..Default::default()
        })
        .with(Crate {})
        .with(BoundingBox(Vec2::new(50.0, 50.0)))
        .with(Velocity(Vec2::zero()));

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.5, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(00.0, 250.0, 1.0)),
            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
            ..Default::default()
        })
        .with(Crate {})
        .with(BoundingBox(Vec2::new(50.0, 50.0)))
        .with(Velocity(Vec2::zero()));
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
                    escalator_transform.translation.y + -escalator_box.0.y / 2.0 + (step.0.y) / 2.0 + (index as f32) * step.0.y,
                0.0,
            )),
            Arm::D,
        ));
    }
    result
}

fn step_intrinsic_velocity(mut query: Query<(&Step, &mut IntrinsicVelocity)>) {
    for (step, mut intrinsic_velocity) in query.iter_mut() {
        match step.arm {
            Arm::A => {
                *intrinsic_velocity = IntrinsicVelocity(Vec2::new(0.0, -1.0));
            }
            Arm::B => {
                *intrinsic_velocity = IntrinsicVelocity(Vec2::new(1.0, -1.0));
            }
            Arm::C => {
                *intrinsic_velocity = IntrinsicVelocity(Vec2::new(1.0, 0.0));
            }
            Arm::D => {
                *intrinsic_velocity = IntrinsicVelocity(Vec2::new(-1.0, 1.0));
            }
        }
    }
}

fn update_position(mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.0.x;
        transform.translation.y += velocity.0.y;
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

fn ground_velocity(mut ungrounded: Query<&mut Velocity, Without<Ground>>) {
    for mut velocity in ungrounded.iter_mut() {
        *velocity = Velocity(Vec2::new(0.0, f32::MIN));
    }
}

// TODO: maybe rewrite this using itertools instead of a QuerySet
// See: https://docs.rs/itertools/0.10.0/itertools/trait.Itertools.html#method.permutations
fn propagate_velocity(
    atop_query: Query<(Entity, &Transform, &BoundingBox), Without<Step>>,
    bases_query: Query<(Entity, &Transform, &BoundingBox), Without<Escalator>>,
    steps: Query<(&Step, Entity)>,

    grounds: Query<(&Ground, Entity)>,

    intrinsic_velocities: Query<&IntrinsicVelocity>,

    mut velocities: Query<&mut Velocity>,
) {
    let mut edges: HashMap<Entity, HashSet<Entity>> = HashMap::new();

    let mut bases: HashSet<Entity> = HashSet::default();
    let mut atops: HashSet<Entity> = HashSet::default();

    for (step, step_entity) in steps.iter() {
        let current_atops = edges.entry(step.escalator).or_insert(HashSet::new());
        current_atops.insert(step_entity);
        atops.insert(step_entity);
        bases.insert(step.escalator);
    }

    for (atop_entity, atop_transform, atop_box) in atop_query.iter() {
        let mut is_atop_anything = false;
        for (below_entity, below_transform, below_box) in bases_query.iter() {
            if is_atop(atop_transform, atop_box, below_transform, below_box) {
                is_atop_anything = true;
                let current_atops = edges.entry(below_entity).or_insert(HashSet::new());

                current_atops.insert(atop_entity);
                atops.insert(atop_entity);
                bases.insert(below_entity);
            }
        }
        if !is_atop_anything {
            bases.insert(atop_entity);
        }
    }

    let roots = bases.difference(&atops);

    for path in build_paths(roots, edges) {
        let base = path.first().expect("first");

        let mut cumulative_velocity = match grounds.get(*base) {
            Ok(_) => Velocity(Vec2::zero()),
            Err(_) => Velocity(Vec2::new(0.0, -1.0)),
        };

        for entity in path.iter() {
            let mut node_velocity = velocities.get_mut(*entity).expect("velocity query");
            // add in intrinsic velocity here

            if let Ok(intrinsic_velocity) = intrinsic_velocities.get(*entity) {
                cumulative_velocity.0.x += intrinsic_velocity.0.x;
                cumulative_velocity.0.y += intrinsic_velocity.0.y;
            }

            // somehow max here

            if cumulative_velocity.0.y > node_velocity.0.y {
                *node_velocity = cumulative_velocity.clone();
            }
        }
    }
}

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

            return result;
        }
        None => return vec![vec![current]],
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
