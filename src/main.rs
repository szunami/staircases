use std::collections::HashMap;

use bevy::{prelude::*, transform::transform_propagate_system::transform_propagate_system};

#[derive(Clone)]
struct BoundingBox(Vec2);

struct Escalator;

fn steps(escalator: &BoundingBox, step: &BoundingBox) -> Vec<(Transform, Arm)> {
    let mut result = vec![];

    // A
    result.push((
        Transform::from_translation(Vec3::new(
            -escalator.0.x / 2.0 + step.0.x / 2.0,
            escalator.0.y / 2.0 - step.0.y / 2.0,
            0.0,
        )),
        Arm::A,
    ));

    // B
    let n = (escalator.0.y / step.0.y) as i32;

    for index in 0..n - 2 {
        result.push((
            Transform::from_translation(Vec3::new(
                -escalator.0.x / 2.0 + step.0.x / 2.0 + index as f32 * step.0.x,
                escalator.0.y / 2.0 - 3.0 * step.0.y / 2.0 - index as f32 * step.0.y,
                0.0,
            )),
            Arm::B,
        ))
    }

    // C
    result.push((
        Transform::from_translation(Vec3::new(
            escalator.0.x / 2.0 - 3.0 * step.0.y / 2.0,
            -escalator.0.y / 2.0 + step.0.y / 2.0,
            0.0,
        )),
        Arm::C,
    ));

    // D
    for index in 0..n - 1 {
        result.push((
            Transform::from_translation(Vec3::new(
                escalator.0.x / 2.0 - step.0.x / 2.0 - (index as f32) * step.0.x,
                -escalator.0.y / 2.0 + (step.0.y) / 2.0 + (index as f32) * step.0.y,
                0.0,
            )),
            Arm::D,
        ));
    }
    result
}

struct Step {
    arm: Arm,
}

enum Arm {
    A,
    B,
    C,
    D,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(step_velocity.system())
        .add_system(gravity_system.system())
        .add_system(update_position.system())
        .add_system(update_step_arm.system())
        .add_system(transform_propagate_system.system())
        .add_system(x_collision_correction.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
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

    let escalator_transform = Transform::from_translation(Vec3::new(100.0, 0.0, 0.0));

    let escalator_box = BoundingBox(Vec2::new(200.0, 200.0));

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
            texture_atlas: walk_handle,
            transform: escalator_transform,
            ..Default::default()
        })
        .with(Escalator {})
        .with_children(|parent| {
            let step_box = BoundingBox(Vec2::new(50.0, 50.0));
            for (step_transform, arm) in steps(&escalator_box, &step_box) {
                parent
                    .spawn(SpriteBundle {
                        material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
                        transform: step_transform,
                        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
                        ..Default::default()
                    })
                    .with(step_box.clone())
                    .with(Step { arm })
                    .with(Velocity(Vec2::zero()));
            }

            // A
        })
        .with(escalator_box);

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.5, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(100.0, 125.0, 1.0)),
            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
            ..Default::default()
        })
        .with(Crate {})
        .with(BoundingBox(Vec2::new(50.0, 50.0)))
        .with(Velocity(Vec2::zero()));

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.5, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(100.0, 175.0, 1.0)),
            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
            ..Default::default()
        })
        .with(Crate {})
        .with(BoundingBox(Vec2::new(50.0, 50.0)))
        .with(Velocity(Vec2::zero()));
}

fn step_velocity(mut query: Query<(&Step, &mut Velocity)>) {
    for (step, mut velocity) in query.iter_mut() {
        match step.arm {
            Arm::A => {
                *velocity = Velocity(Vec2::new(0.0, -1.0));
            }
            Arm::B => {
                *velocity = Velocity(Vec2::new(1.0, -1.0));
            }
            Arm::C => {
                *velocity = Velocity(Vec2::new(1.0, 0.0));
            }
            Arm::D => {
                *velocity = Velocity(Vec2::new(-1.0, 1.0));
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
    parents_query: Query<(Entity, &Children)>,

    mut steps: Query<&mut Step>,
    step_boxes: Query<&BoundingBox>,
    escalators: Query<&BoundingBox>,

    transform_query: Query<&Transform>,
) {
    for (parent, children) in parents_query.iter() {
        let escalator = escalators.get(parent).expect("parent");

        for child in children.iter() {
            let mut step = steps.get_mut(*child).expect("step");
            let step_box = step_boxes.get(*child).expect("step box");

            let step_transform = transform_query.get(*child).expect("step transform");

            let step_top = step_transform.translation.y + step_box.0.y / 2.0;
            let step_bottom = step_transform.translation.y - step_box.0.y / 2.0;
            let step_right = step_transform.translation.x + step_box.0.x / 2.0;

            let escalator_top = escalator.0.y / 2.0;
            let escalator_bottom = -escalator.0.y / 2.0;
            let escalator_right = escalator.0.x / 2.0;

            match step.arm {
                Arm::A => {
                    if step_bottom == escalator_top - 2.0 * step_box.0.y {
                        step.arm = Arm::B;
                    }
                }
                Arm::B => {
                    if step_bottom == escalator_bottom {
                        step.arm = Arm::C;
                    }
                }
                Arm::C => {
                    if step_right == escalator_right {
                        step.arm = Arm::D;
                    }
                }
                Arm::D => {
                    if step_top == escalator_top {
                        step.arm = Arm::A;
                    }
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

    atop_bottom == below_top
        && ((atop_left <= below_left && below_left < atop_right)
            || (below_left <= atop_left && atop_left < below_right))
}

// TODO: maybe rewrite this using itertools instead of a QuerySet
// See: https://docs.rs/itertools/0.10.0/itertools/trait.Itertools.html#method.permutations
fn gravity_system(
    mut crates: QuerySet<(
        Query<(&Crate, Entity, &Transform, &Velocity, &BoundingBox)>,
        Query<(&Crate, Entity, &mut Velocity)>,
    )>,

    steps: Query<(&Step, &GlobalTransform, &Velocity, &BoundingBox)>,
) {
    // arrange items into stacks, apply gravity / transitive velocity up stacks
    // for each crate, is it atop something?

    // HashMap<Entity, Vec<Entity>> for stacks (base -> atop)
    // if A is atop B, enter A to B
    // HashSet<Entity> for bases
    // if A is not atop anything add it to bases

    // need to do math against immutable, then write to mutable

    // let atop_stacks: HashMap<Entity, Vec<Entity>> = HashMap::new();
    // let bases: HashSet<Entity> = HashSet::default();

    let mut crate_velocity: HashMap<Entity, Velocity> = HashMap::new();

    for (
        crate_atop,
        crate_atop_entity,
        crate_atop_transform,
        crate_atop_velocity,
        crate_atop_box,
    ) in crates.q0().iter()
    {
        let mut atop_any = false;

        for (
            _crate_below,
            _crate_below_entity,
            crate_below_transform,
            crate_below_velocity,
            crate_below_box,
        ) in crates.q0().iter()
        {
            if is_atop(
                crate_atop_transform,
                crate_atop_box,
                crate_below_transform,
                crate_below_box,
            ) {
                let current_velocity = crate_velocity
                    .entry(crate_atop_entity)
                    .or_insert(crate_below_velocity.clone());
                if current_velocity.0.y < crate_below_velocity.0.y {
                    *current_velocity = crate_below_velocity.clone();
                }
                atop_any = true;
            }
        }

        // need to handle steps / escalators separately

        for (_step, step_transform, step_velocity, step_box) in steps.iter() {
            if is_atop(
                crate_atop_transform,
                crate_atop_box,
                &Transform::from(*step_transform),
                step_box,
            ) {
                let current_velocity = crate_velocity
                    .entry(crate_atop_entity)
                    .or_insert(step_velocity.clone());
                if current_velocity.0.y < step_velocity.0.y {
                    *current_velocity = step_velocity.clone();
                }

                atop_any = true;
            }
        }

        if !atop_any {
            crate_velocity.insert(crate_atop_entity, Velocity(Vec2::new(0.0, -1.0)));
        }
    }
    for (_crate, crate_entity, mut velocity) in crates.q1_mut().iter_mut() {
        *velocity = crate_velocity
            .get(&crate_entity)
            .expect("crate velocity")
            .clone();
    }
}

#[derive(Clone)]
struct Velocity(Vec2);

#[derive(PartialEq, Eq, Hash)]
struct Crate {}

fn x_collision_correction(
    mut crates: Query<(&Crate, &mut Transform, &BoundingBox)>,
    steps: Query<(&Step, &GlobalTransform, &BoundingBox)>,
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
