use bevy::prelude::*;




struct Step {
    escalator_height: f32,
    escalator_width: f32,
    step_height: f32,
    step_width: f32,
    arm: Arm,
}

enum Arm {
    A, B, C, D
}

impl Default for Step {
    fn default() -> Self {
        Step {
            step_height: 50.0,
            step_width: 50.0,
            escalator_height: 150.0,
            escalator_width: 150.0,
            arm: Arm::A,
        }
    }
}


fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())

        .add_system(step.system())

        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}

fn setup(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default());

    commands.spawn(SpriteBundle {
        material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
        transform: Transform::from_translation(Vec3::new(0.0, 150.0, 0.0)),
        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
        ..Default::default()
    }).with(Step::default());

    commands.spawn(SpriteBundle {
        material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
        transform: Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
        ..Default::default()
    }).with(Step{
        arm: Arm::B,
        ..Step::default()
    });

    commands.spawn(SpriteBundle {
        material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
        transform: Transform::from_translation(Vec3::new(50.0, 50.0, 0.0)),
        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
        ..Default::default()
    }).with(Step{
        arm: Arm::B,
        ..Step::default()
    });

    commands.spawn(SpriteBundle {
        material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
        transform: Transform::from_translation(Vec3::new(100.0, 0.0, 00.0)),
        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
        ..Default::default()
    }).with(Step{
        arm: Arm::C,
        ..Step::default()
    });

    commands.spawn(SpriteBundle {
        material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
        transform: Transform::from_translation(Vec3::new(150.0, 0.0, 00.0)),
        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
        ..Default::default()
    }).with(Step{
        arm: Arm::D,
        ..Step::default()
    });

    commands.spawn(SpriteBundle {
        material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
        transform: Transform::from_translation(Vec3::new(100.0, 50.0, 00.0)),
        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
        ..Default::default()
    }).with(Step{
        arm: Arm::D,
        ..Step::default()
    });

    commands.spawn(SpriteBundle {
        material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
        transform: Transform::from_translation(Vec3::new(100.0, 50.0, 00.0)),
        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
        ..Default::default()
    }).with(Step{
        arm: Arm::D,
        ..Step::default()
    });

    commands.spawn(SpriteBundle {
        material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
        transform: Transform::from_translation(Vec3::new(50.0, 100.0, 00.0)),
        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
        ..Default::default()
    }).with(Step{
        arm: Arm::D,
        ..Step::default()
    }); 
}


fn step(
    mut query: Query<(&mut Step, &mut Transform)>
) {
    for (mut step, mut transform) in query.iter_mut() {

        // update transform

        match step.arm {
            Arm::A => {
                transform.translation.y -= 1.0;
            }
            Arm::B => {
                transform.translation.x += 1.0;
                transform.translation.y -= 1.0;
            }
            Arm::C => {
                transform.translation.x += 1.0;
            }
            Arm::D => {
                transform.translation.x -= 1.0;
                transform.translation.y += 1.0;
            }
        }

        // update arm
        match step.arm {
            Arm::A => {
                if transform.translation.y == step.escalator_height - step.step_height {
                    step.arm = Arm::B;
                }
            }
            Arm::B => {
                if transform.translation.y == 0.0 {
                    step.arm = Arm::C;
                }
            }
            Arm::C => {
                if transform.translation.x == step.escalator_width {
                    step.arm = Arm::D;
                }
            }
            Arm::D => {
                if transform.translation.y == step.escalator_height {
                    step.arm = Arm::A;
                }
            }
        }


        dbg!(transform.translation);
    }
}