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
        .add_system(escalator.system())
        .add_system(step.system())
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
                .with(Step::default());

            // B
            parent
                .spawn(SpriteBundle {
                    material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
                    transform: Transform::from_translation(Vec3::new(-75.0, 25.0, 1.0)),
                    sprite: Sprite::new(Vec2::new(50.0, 50.0)),
                    ..Default::default()
                })
                .with(Step {
                    arm: Arm::B,
                    ..Step::default()
                });
        });
}

fn escalator(mut escalators: Query<(&Escalator, &mut Transform)>) {
    for (_escalator, mut transform) in escalators.iter_mut() {
        transform.translation.x += 1.0;
        transform.translation.y -= 1.0;
    }
}

fn step(
    parents_query: Query<(Entity, &Children)>,

    mut steps: Query<&mut Step>,
    escalators: Query<&Escalator>,

    mut transform_query: Query<&mut Transform>,
) {
    for (parent, children) in parents_query.iter() {
        let escalator = escalators.get(parent).expect("parent");

        for child in children.iter() {
            let mut step = steps.get_mut(*child).expect("step");

            let mut step_transform = transform_query.get_mut(*child).expect("step transform");

            match step.arm {
                Arm::A => {
                    step_transform.translation.y -= 1.0;
                }
                Arm::B => {
                    step_transform.translation.x += 1.0;
                    step_transform.translation.y -= 1.0;
                }
                Arm::C => {
                    step_transform.translation.x += 1.0;
                }
                Arm::D => {
                    step_transform.translation.x -= 1.0;
                    step_transform.translation.y += 1.0;
                }
            }

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
