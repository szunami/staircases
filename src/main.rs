use std::collections::{HashMap, HashSet};

use bevy::{diagnostic::Diagnostics, prelude::*};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_startup_system(setup.system())
        // .add_startup_system(setup2.system())
        // .add_system(framerate.system())
        .add_system(step_intrinsic_velocity.system())
        .add_system(player_intrinsic_velocity.system())
        .add_system(reset_ungrounded_velocity.system())
        .add_system(propagate_velocity_horizontally.system())
        .add_system(propagate_velocity_vertically.system())
        .add_system(update_position.system())
        .add_system(update_step_arm.system())
        .add_system(x_collision_correction.system())
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
struct Velocity(Vec2);

struct IntrinsicVelocity(Vec2);

struct Ground;

#[derive(PartialEq, Eq, Hash)]
struct Crate;

struct Player;

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

    {
        let escalator_transform = Transform::from_translation(Vec3::new(100.0, 0.0, 0.0));

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
            .with(Velocity(Vec2::zero()));
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
            .with(Velocity(Vec2::zero()));
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

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            transform: Transform::from_translation(Vec3::new(00.0, 350.0, 1.0)),
            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
            ..Default::default()
        })
        .with(Player {})
        .with(BoundingBox(Vec2::new(50.0, 50.0)))
        .with(Velocity(Vec2::zero()))
        .with(IntrinsicVelocity(Vec2::zero()));
}

fn setup2(
    commands: &mut Commands,

    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,

    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let walk_handle = asset_server.load("textures/base.png");
    let walk_atlas = TextureAtlas::from_grid(walk_handle, Vec2::new(200.0, 200.0), 1, 1);

    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default());

    {
        let ground_box = Vec2::new(400.0, 50.0);
        commands
            .spawn(SpriteBundle {
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(0.0, -50.0, 1.0)),
                sprite: Sprite::new(ground_box),
                ..Default::default()
            })
            .with(Ground {})
            .with(BoundingBox(ground_box))
            .with(Velocity(Vec2::zero()));
    }

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.5, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(60.0, 0.0, 1.0)),
            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
            ..Default::default()
        })
        .with(Crate {})
        .with(BoundingBox(Vec2::new(50.0, 50.0)))
        .with(Velocity(Vec2::zero()));
        // commands
        // .spawn(SpriteBundle {
        //     material: materials.add(Color::rgb(1.0, 0.5, 1.0).into()),
        //     transform: Transform::from_translation(Vec3::new(100.0, 0.0, 1.0)),
        //     sprite: Sprite::new(Vec2::new(50.0, 50.0)),
        //     ..Default::default()
        // })
        // .with(Crate {})
        // .with(BoundingBox(Vec2::new(50.0, 50.0)))
        // .with(Velocity(Vec2::zero()));

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
            ..Default::default()
        })
        .with(Player {})
        .with(BoundingBox(Vec2::new(50.0, 50.0)))
        .with(Velocity(Vec2::zero()))
        .with(IntrinsicVelocity(Vec2::zero()));
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
    mut query: Query<(&Player, &mut IntrinsicVelocity)>,
) {
    for (_player, mut velocity) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            velocity.0.x = -1.0;
        } else if keyboard_input.pressed(KeyCode::D) {
            velocity.0.x = 1.0;
        } else {
            velocity.0.x = 0.0;
        }
    }
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

        if (velocity.0.x > 0.0) {
            dbg!(transform.translation);
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

fn reset_ungrounded_velocity(mut ungrounded: Query<&mut Velocity, Without<Ground>>) {
    for mut velocity in ungrounded.iter_mut() {
        *velocity = Velocity(Vec2::new(0.0, f32::MIN));
    }
}


fn propagate_velocity_horizontally(
    query: Query<(Entity, &Transform, &BoundingBox)>,
    intrinsic_velocities: Query<&IntrinsicVelocity>,

    mut velocities: Query<&mut Velocity>,
){
    // might want to link steps / staircases?
    // maintain L edges and R edges?

    let mut left_edges: HashMap<Entity, HashSet<Entity>> = HashMap::new();
    let mut left_bases: HashSet<Entity> = HashSet::new();
    let mut left_nonbases: HashSet<Entity> = HashSet::new();

    for (left_entity, left_transform, left_box) in query.iter() {
        for (right_entity, right_transform, right_box) in query.iter() {
            if is_beside(left_transform, left_box, right_transform, right_box) {
                dbg!("beside!");
                let current_lefts = left_edges.entry(left_entity).or_insert(HashSet::new());

                current_lefts.insert(right_entity);
                left_bases.insert(left_entity);
                left_nonbases.insert(right_entity);
            }
        }
    }

    let left_roots = left_bases.difference(&left_nonbases);

    let paths = build_paths(left_roots, left_edges);

    if paths.len() != 0 {
        dbg!(paths.len());
    }

    for path in paths {
        let mut max_velocity_so_far: f32 = 0.0;

        for entity in path.iter() {
            if let Ok(intrinsic_velocity) = intrinsic_velocities.get(*entity) {
                max_velocity_so_far = max_velocity_so_far.max(intrinsic_velocity.0.x);
            }

            let mut node_velocity = velocities.get_mut(*entity).expect("velocity");
            node_velocity.0.x = max_velocity_so_far;

            // dbg!(node_velocity.0);

        }


    }
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


// TODO: maybe rewrite this using itertools instead of a QuerySet
// See: https://docs.rs/itertools/0.10.0/itertools/trait.Itertools.html#method.permutations
fn propagate_velocity_vertically(
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
        let mut cumulative_velocity = Velocity(Vec2::new(0.0, -1.0));

        for entity in path.iter() {
            // want to recheck grounding at each layer

            if let Ok(_) = grounds.get(*entity) {
                cumulative_velocity = Velocity(Vec2::zero());
            };

            let mut node_velocity = velocities.get_mut(*entity).expect("velocity query");
            // add in intrinsic velocity here

            if let Ok(intrinsic_velocity) = intrinsic_velocities.get(*entity) {
                cumulative_velocity.0.x += intrinsic_velocity.0.x;
                cumulative_velocity.0.y += intrinsic_velocity.0.y;
            }

            // somehow max here

            if cumulative_velocity.0.y > node_velocity.0.y {
                // could have x velocity from prior propagation
                node_velocity.0.x += cumulative_velocity.0.x;

                node_velocity.0.y = cumulative_velocity.0.y;
            }
            // dbg!(node_velocity.0);
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
