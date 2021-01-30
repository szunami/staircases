use bevy::prelude::*;

struct Escalator {
    escalator_height: f32,
    escalator_width: f32,
}

impl Default for Escalator {
    fn default() -> Self {
        Escalator {
            escalator_height: 200.0,
            escalator_width: 200.0,
        }
    }
}

struct Step {
    step_height: f32,
    step_width: f32,
    arm: Arm,
}

enum Arm {
    A,
    B,
    C,
    D,
}

impl Default for Step {
    fn default() -> Self {
        Step {
            step_height: 50.0,
            step_width: 50.0,
            arm: Arm::A,
        }
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(step_velocity.system())
        .add_system(crate_system.system())
        .add_system(velocity.system())
        .add_system(step_arm.system())
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

    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: walk_handle,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .with(Escalator::default())
        .with_children(|parent| {
            // A
            parent
                .spawn(SpriteBundle {
                    material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
                    transform: Transform::from_translation(Vec3::new(-75.0, 75.0, 1.0)),
                    sprite: Sprite::new(Vec2::new(50.0, 50.0)),
                    ..Default::default()
                })
                .with(Step::default())
                .with(Velocity(Vec2::zero()));
        });

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.5, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(-75.0, 125.0, 1.0)),
            sprite: Sprite::new(Vec2::new(50.0, 50.0)),
            ..Default::default()
        })
        .with(Crate {
            width: 50.0,
            height: 50.0,
        })
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

fn velocity(mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.0.x;
        transform.translation.y += velocity.0.y;
    }
}

fn step_arm(
    parents_query: Query<(Entity, &Children)>,

    mut steps: Query<&mut Step>,
    escalators: Query<&Escalator>,

    transform_query: Query<&Transform>,
) {
    for (parent, children) in parents_query.iter() {
        let escalator = escalators.get(parent).expect("parent");

        for child in children.iter() {
            let mut step = steps.get_mut(*child).expect("step");

            let step_transform = transform_query.get(*child).expect("step transform");

            let step_top = step_transform.translation.y + step.step_height / 2.0;
            let step_bottom = step_transform.translation.y - step.step_height / 2.0;
            let step_right = step_transform.translation.x + step.step_width / 2.0;

            let escalator_top = escalator.escalator_height / 2.0;
            let escalator_bottom = -escalator.escalator_height / 2.0;
            let escalator_right = escalator.escalator_width / 2.0;

            match step.arm {
                Arm::A => {
                    if step_bottom == escalator_top - 2.0 * step.step_height {
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

fn crate_system(
    mut crates: Query<(&Crate, &Transform, &mut Velocity)>,
    steps: Query<(&Step, &Transform, &Velocity)>,
) {
    for (cate, crate_transform, mut crate_velocity) in crates.iter_mut() {
        let mut atop = false;

        let crate_bottom = crate_transform.translation.y - cate.height / 2.0;

        for (step, step_transform, mut step_velocity) in steps.iter() {
            let step_top = step_transform.translation.y + step.step_height / 2.0;

            if step_top == crate_bottom {
                *crate_velocity = step_velocity.clone();
                atop = true;
                dbg!("atop");
            }
        }

        if !atop {
            crate_velocity.0.y = -1.0;
        }
    }
}

#[derive(Clone)]
struct Velocity(Vec2);

struct Crate {
    width: f32,
    height: f32,
}
