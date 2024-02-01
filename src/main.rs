use bevy::{
    app::{App, PluginGroup, PreUpdate, Startup, Update},
    asset::{Asset, AssetMetaCheck, Assets, Handle},
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        schedule::{IntoSystemConfigs, IntoSystemSet},
        system::{Commands, Query, Res, ResMut, Resource, System},
    },
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    input::{mouse::MouseButton, Input},
    math::{Mat2, Quat, Vec2, Vec3, Vec4},
    pbr::{MaterialMeshBundle, MaterialPlugin},
    prelude::{Deref, DerefMut},
    reflect::TypePath,
    render::{
        camera::{Camera, OrthographicProjection},
        mesh::{shape, Mesh},
        render_resource::AsBindGroup,
    },
    time::Time,
    transform::{
        components::{GlobalTransform, Transform},
        TransformBundle,
    },
    window::{PrimaryWindow, Window, WindowPlugin},
    DefaultPlugins,
};
use std::{f32, marker::PhantomData};
use stopwatch::{StopwatchComponent, StopwatchComponentPlugin};
use traveller::TravellerPlugin;

#[derive(Component)]
struct PrimaryCamera;

pub struct WalkingAttribute {
    pub initial: f32,
    pub rampup: Option<f32>,
}
#[derive(Component)]
struct WalkingAttributes {
    walking_speed: WalkingAttribute,
    rotation_speed: Option<WalkingAttribute>,
}

#[derive(Component, Deref, DerefMut)]
struct PlayerPositionDestination(Entity);

#[derive(Component)]
struct Player;

mod traveller {
    use std::marker::PhantomData;

    use bevy::{
        app::{Plugin, Update},
        ecs::{component::Component, schedule::IntoSystemConfigs, system::IsFunctionSystem},
    };
    pub struct TravellerPlugin<
        ReachDestinationSystemMarker,
        ReachDestinationSystemConfigs: IntoSystemConfigs<ReachDestinationSystemMarker>,
        ReachedDestinationSystemMarker,
        ReachedDestinationSystemConfigs: IntoSystemConfigs<ReachedDestinationSystemMarker>,
    > {
        reach_destination_system: ReachDestinationSystemConfigs,
        reached_destination_system: Option<ReachedDestinationSystemConfigs>,
        _phantom_data: PhantomData<(ReachDestinationSystemMarker, ReachedDestinationSystemMarker)>,
    }

    impl<
            ReachDestinationSystemMarker: std::marker::Sync + std::marker::Send + 'static,
            ReachDestinationSystemConfigs: IntoSystemConfigs<ReachDestinationSystemMarker>
                + std::marker::Sync
                + std::marker::Send
                + 'static
                + Clone,
            ReachedDestinationSystemMarker: std::marker::Sync + std::marker::Send + 'static,
            ReachedDestinationSystemConfigs: IntoSystemConfigs<ReachedDestinationSystemMarker>
                + std::marker::Sync
                + std::marker::Send
                + 'static
                + Clone,
        > Plugin
        for TravellerPlugin<
            ReachDestinationSystemMarker,
            ReachDestinationSystemConfigs,
            ReachedDestinationSystemMarker,
            ReachedDestinationSystemConfigs,
        >
    {
        fn build(&self, app: &mut bevy::prelude::App) {
            app.add_systems(Update, (self.reach_destination_system.clone(),));
            if let Some(reached_destination_system) = &self.reached_destination_system {
                app.add_systems(Update, reached_destination_system.clone());
            }
        }
    }

    impl<
            ReachDestinationSystemMarker,
            ReachDestinationSystemConfigs: IntoSystemConfigs<ReachDestinationSystemMarker>,
            ReachedDestinationSystemMarker,
            ReachedDestinationSystemConfigs: IntoSystemConfigs<ReachedDestinationSystemMarker>,
        >
        TravellerPlugin<
            ReachDestinationSystemMarker,
            ReachDestinationSystemConfigs,
            ReachedDestinationSystemMarker,
            ReachedDestinationSystemConfigs,
        >
    {
        pub fn new_reached(
            reach_destination_system: ReachDestinationSystemConfigs,
            reached_destination_system: ReachedDestinationSystemConfigs,
        ) -> Self {
            Self {
                reach_destination_system,
                reached_destination_system: Some(reached_destination_system),
                _phantom_data: PhantomData,
            }
        }
    }

    impl<
            ReachDestinationSystemMarker,
            ReachDestinationSystemConfigs: IntoSystemConfigs<ReachDestinationSystemMarker>,
        >
        TravellerPlugin<
            ReachDestinationSystemMarker,
            ReachDestinationSystemConfigs,
            (IsFunctionSystem, fn()),
            fn(),
        >
    {
        pub fn new(reach_destination_system: ReachDestinationSystemConfigs) -> Self {
            Self {
                reach_destination_system,
                reached_destination_system: None,
                _phantom_data: PhantomData,
            }
        }
    }
}

#[derive(Resource)]
struct GlobalResources {
    ball: Handle<Mesh>,
    white_mat: Handle<DefaultMaterial>,
    red_mat: Handle<DefaultMaterial>,
}

mod stopwatch {
    use bevy::{
        app::{Plugin, PostUpdate},
        ecs::{
            component::Component,
            system::{Query, Res},
        },
        prelude::{Deref, DerefMut},
        time::{Stopwatch, Time},
    };

    #[derive(Component, Default, Deref, DerefMut)]
    pub struct StopwatchComponent(Stopwatch);

    pub struct StopwatchComponentPlugin;
    fn update_stopwatches(mut stopwatches: Query<&mut StopwatchComponent>, time: Res<Time>) {
        stopwatches.par_iter_mut().for_each(|mut stop_watch| {
            stop_watch.0.tick(time.delta());
        });
    }
    impl Plugin for StopwatchComponentPlugin {
        fn build(&self, app: &mut bevy::prelude::App) {
            app.add_systems(PostUpdate, update_stopwatches);
        }
    }
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
            TravellerPlugin::new(
                |travellers: Query<
                    (
                        Entity,
                        &WalkingAttributes,
                        &PlayerPositionDestination,
                        &StopwatchComponent,
                    ),
                    With<Player>,
                >,
                 mut q_transforms: Query<&mut Transform>,
                 time: Res<Time>,
                 mut commands: Commands| {
                    for (traveller, walking_attributes, destination, elapased_time) in
                        travellers.into_iter()
                    {
                        let [mut trav_transform, dest_transform] = q_transforms
                            .get_many_mut([traveller, **destination])
                            .unwrap();

                        let difference =
                            Vec2::new(trav_transform.translation.x, trav_transform.translation.z)
                                - Vec2::new(
                                    dest_transform.translation.x,
                                    dest_transform.translation.z,
                                );

                        let distance = difference.length();
                        let step_size = match walking_attributes.walking_speed.rampup {
                            Some(rampup) => {
                                walking_attributes.walking_speed.initial
                                    + rampup * elapased_time.elapsed().as_secs_f32()
                            }
                            None => walking_attributes.walking_speed.initial,
                        } * time.delta().as_secs_f32();

                        let mut difference_angle = f32::atan2(difference.x, difference.y);

                        let rotation_step_size =
                            walking_attributes
                                .rotation_speed
                                .as_ref()
                                .map(|rotation_speed| {
                                    (match rotation_speed.rampup {
                                        Some(rampup) => {
                                            rotation_speed.initial
                                                + rampup * elapased_time.elapsed().as_secs_f32()
                                        }
                                        None => rotation_speed.initial,
                                    }) * std::f32::consts::PI
                                        * time.delta().as_secs_f32()
                                });

                        if let Some(rotation_step_size) = rotation_step_size {
                            let player_rot = trav_transform
                                .rotation
                                .to_euler(bevy::math::EulerRot::XYZ)
                                .1;
                            if player_rot != difference_angle {
                                if (difference_angle) > std::f32::consts::PI {
                                    difference_angle -= std::f32::consts::PI * 2.0;
                                } else if difference_angle < -std::f32::consts::PI {
                                    difference_angle += std::f32::consts::PI * 2.0;
                                }
                            }
                        } else {
                            trav_transform.rotation = Quat::from_euler(
                                bevy::math::EulerRot::YXZ,
                                difference_angle,
                                0.0,
                                0.0,
                            );
                            let player_rot = trav_transform
                                .rotation
                                .to_euler(bevy::math::EulerRot::YXZ)
                                .0;
                        }
                        if distance <= step_size {
                            commands
                                .entity(traveller)
                                .remove::<(StopwatchComponent, PlayerPositionDestination)>();
                            trav_transform.translation = dest_transform.translation;
                            /*
                            if !(travelling.ignore_rotation) {
                                trav_transform.rotation = dest_transform.rotation;
                            }
                            */
                            commands.entity(**destination).despawn_recursive();

                            return;
                        }

                        let walk_vector = trav_transform.forward() * step_size;
                        trav_transform.translation += walk_vector;
                    }
                },
            ),
            StopwatchComponentPlugin,
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
                commands.insert_resource(GlobalResources {
                    ball: mesh.clone(),
                    white_mat: material.clone(),
                    red_mat: materials.add(DefaultMaterial {
                        color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                    }),
                });

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
                    .spawn((
                        TransformBundle {
                            global: GlobalTransform::default(),
                            local: Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
                        },
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
                 mut q_destinations: Query<&mut Transform>,
                 mut q_players: Query<(Entity, Option<&PlayerPositionDestination>)>,
                 mouse_buttons: Res<Input<MouseButton>>,
                 mut commands: Commands,
                 global_resources: Res<GlobalResources>| {
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
                            let cursor_pos = Vec3::new(cursor_pos.x, 0.0, cursor_pos.y);
                            for (player, destination) in q_players.into_iter() {
                                match destination {
                                    Some(pos) => {
                                        q_destinations.get_mut(**pos).unwrap().translation =
                                            cursor_pos
                                    }
                                    None => {
                                        let destination = commands
                                            .spawn((MaterialMeshBundle {
                                                material: global_resources.red_mat.clone(),
                                                mesh: global_resources.ball.clone(),
                                                transform: Transform {
                                                    translation: cursor_pos,
                                                    scale: Vec3::splat(0.125),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },))
                                            .id();
                                        commands.entity(player).insert((
                                            StopwatchComponent::default(),
                                            PlayerPositionDestination(destination),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                },
            ),
        )
        /*
        .add_systems(
            Update,
            ,
        )
        */
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
