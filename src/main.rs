use bevy::{
    app::{App, PluginGroup, PreUpdate, Startup, Update},
    asset::{Asset, AssetMetaCheck, Assets},
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::BuildChildren,
    input::{mouse::MouseButton, Input},
    log::info,
    math::{Mat2, Quat, Vec2, Vec3, Vec4},
    pbr::{MaterialMeshBundle, MaterialPlugin},
    reflect::TypePath,
    render::{
        camera::{Camera, OrthographicProjection},
        mesh::{shape, Mesh},
        render_resource::AsBindGroup,
    },
    time::Time,
    transform::{components::Transform, TransformBundle},
    window::{PrimaryWindow, Window, WindowPlugin},
    DefaultPlugins,
};
use std::{f32, ops::AddAssign, time::Duration};

#[derive(Component)]
struct PrimaryCamera;

#[derive(Component)]
struct TestMouse;

pub struct WalkingAttribute {
    pub initial: f32,
    pub rampup: Option<f32>,
}
#[derive(Component)]
struct WalkingAttributes {
    walking_speed: WalkingAttribute,
    rotation_speed: Option<WalkingAttribute>,
}

#[derive(Component)]
struct Travelling {
    time_spent: Duration,
    destination: Destination,
}
enum Destination {
    Position(Vec2),
}
#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    fit_canvas_to_parent: true,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            MaterialPlugin::<DefaultMaterial>::default(),
        ))
        .add_systems(
            Startup,
            |mut commands: Commands,
             mut meshes: ResMut<Assets<Mesh>>,
             mut materials: ResMut<Assets<DefaultMaterial>>| {
                let mesh = meshes.add(
                    shape::Icosphere {
                        radius: 0.125,
                        subdivisions: 2,
                    }
                    .try_into()
                    .unwrap(),
                );
                let material = materials.add(DefaultMaterial {
                    color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                });
                commands.spawn((
                    MaterialMeshBundle {
                        material: materials.add(DefaultMaterial {
                            color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                        }),
                        mesh: mesh.clone(),
                        visibility: bevy::render::view::Visibility::Hidden,
                        transform: Transform {
                            scale: Vec3::splat(0.125),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    TestMouse,
                ));

                commands
                    .spawn((
                        TransformBundle::default(),
                        Player,
                        WalkingAttributes {
                            walking_speed: WalkingAttribute {
                                initial: 0.5,
                                rampup: Some(2.0),
                            },
                            rotation_speed: None, /*
                                                  Some(WalkingAttribute {
                                                      initial: 0.001,
                                                      rampup: None,
                                                  }),
                                                   */
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn(MaterialMeshBundle {
                            material: material.clone(),
                            mesh: mesh.clone(),
                            transform: Transform {
                                translation: Vec3::new(0.125, 0.0, 0.0),
                                scale: Vec3::splat(0.5),
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                        parent.spawn(MaterialMeshBundle {
                            material: material.clone(),
                            mesh: mesh.clone(),
                            transform: Transform {
                                translation: Vec3::new(-0.125, 0.0, 0.0),
                                scale: Vec3::splat(0.5),
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                        parent.spawn(MaterialMeshBundle {
                            material: material.clone(),
                            mesh: mesh.clone(),
                            transform: Transform {
                                translation: Vec3::new(0.0, 0.25, 0.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                        parent.spawn(MaterialMeshBundle {
                            material: material.clone(),
                            mesh: mesh.clone(),
                            transform: Transform {
                                translation: Vec3::new(0.0, 0.5, 0.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                    });

                commands
                    .spawn(TransformBundle {
                        local: Transform {
                            rotation: Quat::from_euler(
                                bevy::math::EulerRot::YXZ,
                                -f32::consts::FRAC_PI_4,
                                -f32::consts::FRAC_PI_4,
                                0.0,
                            ),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            Camera3dBundle {
                                transform: Transform {
                                    translation: Vec3 {
                                        x: 0.0,
                                        y: 0.0,
                                        z: 3.0,
                                    },
                                    ..Default::default()
                                },
                                projection: bevy::render::camera::Projection::Orthographic(
                                    OrthographicProjection {
                                        near: 0.0,
                                        far: 1000.0,
                                        scale: 4.0,
                                        scaling_mode:
                                            bevy::render::camera::ScalingMode::FixedVertical(1.0),
                                        ..Default::default()
                                    },
                                ),
                                ..Default::default()
                            },
                            PrimaryCamera,
                        ));
                    });
            },
        )
        .add_systems(
            PreUpdate,
            (
                |q_primary_window: Query<&Window, With<PrimaryWindow>>,
                 q_primary_camera: Query<&Camera, With<PrimaryCamera>>,
                 mut q_player_destination: Query<
                    (Entity, Option<&mut Travelling>),
                    With<Player>,
                >,
                 mouse_buttons: Res<Input<MouseButton>>,
                 mut commands: Commands| {
                    if mouse_buttons.pressed(MouseButton::Left) {
                        let primary_window = q_primary_window.single();
                        if let Some(mut cursor_pos) = primary_window.cursor_position() {
                            cursor_pos -= Vec2::from((
                                primary_window.resolution.width(),
                                primary_window.resolution.height(),
                            )) * 0.5;
                            cursor_pos /= primary_window.resolution.height();
                            cursor_pos *= 4.0;
                            cursor_pos.y *= f32::consts::SQRT_2;

                            cursor_pos =
                                Mat2::from_angle(f32::consts::FRAC_PI_4).mul_vec2(cursor_pos);
                            let cursor_pos = Destination::Position(cursor_pos);

                            let (player, player_destination) = q_player_destination.single_mut();
                            match player_destination {
                                Some(mut pos) => pos.destination = cursor_pos,
                                None => {
                                    commands.entity(player).insert(Travelling {
                                        time_spent: Duration::ZERO,
                                        destination: cursor_pos,
                                    });
                                }
                            }
                        }
                    }
                },
            ),
        )
        .add_systems(
            Update,
            |mut travellers: Query<(
                Entity,
                &mut Transform,
                &WalkingAttributes,
                &mut Travelling,
            )>,
             time: Res<Time>,
             mut commands: Commands| {
                travellers.iter_mut().for_each(
                    |(traveller, mut transform, walking_attribute, mut travelling)| {
                        let Travelling {
                            time_spent,
                            destination,
                        } = &mut *travelling;

                        match destination {
                            Destination::Position(pos) => {
                                let difference =
                                    Vec2::new(transform.translation.x, transform.translation.z)
                                        - *pos;

                                let distance = difference.length();
                                let step_size = match walking_attribute.walking_speed.rampup {
                                    Some(rampup) => {
                                        walking_attribute.walking_speed.initial
                                            + rampup * time_spent.as_secs_f32()
                                    }
                                    None => walking_attribute.walking_speed.initial,
                                } * time.delta().as_secs_f32();

                                let mut difference_angle = f32::atan2(difference.x, difference.y);

                                let rotation_step_size = walking_attribute
                                    .rotation_speed
                                    .as_ref()
                                    .map(|rotation_speed| {
                                        (match rotation_speed.rampup {
                                            Some(rampup) => {
                                                rotation_speed.initial
                                                    + rampup * time_spent.as_secs_f32()
                                            }
                                            None => rotation_speed.initial,
                                        }) * std::f32::consts::PI
                                            * time.delta().as_secs_f32()
                                    });

                                if let Some(rotation_step_size) = rotation_step_size {
                                    let player_rot =
                                        transform.rotation.to_euler(bevy::math::EulerRot::XYZ).1;
                                    if player_rot != difference_angle {
                                        if (difference_angle) > std::f32::consts::PI {
                                            difference_angle -= std::f32::consts::PI * 2.0;
                                        } else if difference_angle < -std::f32::consts::PI {
                                            difference_angle += std::f32::consts::PI * 2.0;
                                        }
                                    }
                                } else {
                                    transform.rotation = Quat::from_euler(
                                        bevy::math::EulerRot::YXZ,
                                        difference_angle,
                                        0.0,
                                        0.0,
                                    );
                                    let player_rot =
                                        transform.rotation.to_euler(bevy::math::EulerRot::YXZ).0;
                                    info!(
                                        "{}, {difference_angle}, {player_rot}",
                                        difference_angle == player_rot
                                    );
                                }
                                if distance <= step_size {
                                    commands.entity(traveller).remove::<Travelling>();
                                    transform.translation = Vec3::new(pos.x, 0.0, pos.y);

                                    return;
                                }

                                let walk_vector = transform.forward() * step_size;
                                transform.translation += walk_vector;

                                time_spent.add_assign(time.delta());
                            }
                        };
                    },
                );
            },
        )
        .run()
}
#[derive(Clone, Copy, AsBindGroup, Asset, TypePath)]
struct DefaultMaterial {
    #[uniform(0)]
    color: Vec4,
}

impl bevy::pbr::Material for DefaultMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/default_material.wgsl".into()
    }
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        Self::vertex_shader()
    }
}
