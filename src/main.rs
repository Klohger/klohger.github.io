use bevy::{
    app::{App, PluginGroup, PreUpdate, Startup, Update},
    asset::{Asset, AssetMetaCheck, Assets},
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        component::Component,
        event::{Event, EventReader, EventWriter},
        query::With,
        schedule::States,
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::BuildChildren,
    input::{mouse::MouseButton, Input},
    math::{Mat2, Quat, Vec2, Vec3, Vec4},
    pbr::{MaterialMeshBundle, MaterialPlugin},
    reflect::TypePath,
    render::{
        camera::{Camera, OrthographicProjection},
        mesh::{shape, Mesh},
        render_resource::AsBindGroup,
        view::Visibility,
    },
    transform::{components::Transform, TransformBundle},
    window::{PrimaryWindow, Window, WindowPlugin},
    DefaultPlugins,
};
use std::{default, f32};

#[derive(Component)]
struct Player {
    speed: f32,
    rotation_speed: f32,
    state: PlayerState,
}

#[derive(Component)]
struct PrimaryCamera;

#[derive(Component)]
struct TestMouse;

#[derive(Event, Default, Clone, Copy)]
enum PlayerState {
    #[default]
    Idle,
    GotoPos(Vec2),
    GotoInteractable(()),
}

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
        .add_event::<PlayerState>()
        .add_systems(
            Startup,
            |mut commands: Commands,
             mut meshes: ResMut<Assets<Mesh>>,
             mut materials: ResMut<Assets<DefaultMaterial>>| {
                let mesh = meshes.add(
                    shape::Icosphere {
                        radius: 0.125,
                        subdivisions: 4,
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
                        Player {
                            speed: 0.125,
                            rotation_speed: f32::consts::FRAC_PI_8,
                            state: PlayerState::default(),
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
                 mut player_events: EventWriter<PlayerState>,
                 mouse_buttons: Res<Input<MouseButton>>| {
                    if mouse_buttons.just_pressed(MouseButton::Left) {
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

                            player_events.send(PlayerState::GotoPos(cursor_pos));
                        }
                    }
                },
                |mut q_player: Query<&mut Player>, mut player_events: EventReader<PlayerState>| {
                    if let Some(player_state) = player_events.read().last() {
                        q_player.single_mut().state = *player_state;
                    }
                },
            ),
        )
        .add_systems(
            Update,
            (
                |mut q_test_mouse: Query<(&mut Transform, &mut Visibility), With<TestMouse>>,
                 mut player_events: EventReader<PlayerState>| {
                    let (mut transform, mut visibility) = q_test_mouse.single_mut();
                    for player_event in player_events.read() {
                        match player_event {
                            PlayerState::Idle => *visibility = Visibility::Hidden,
                            PlayerState::GotoPos(pos) => {
                                *visibility = Visibility::Visible;
                                transform.translation = Vec3::from((pos.x, 0.0, pos.y));
                            }
                            PlayerState::GotoInteractable(_) => todo!(),
                        }
                    }
                },
                |mut q_player: Query<(&mut Transform, &mut Player)>| {
                    let (transform, player) = q_player.single();

                    match player.state {
                        PlayerState::Idle => (),
                        PlayerState::GotoPos(pos) => {
                            let player_pos = Vec2::new(transform.translation.x, transform.translation.z);
                            
                        },
                        PlayerState::GotoInteractable(_) => todo!(),
                    }

                },
            ),
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
