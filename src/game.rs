use crate::builder::builders::{arc, line};
use crate::builder::{CurveBuilder, Joint2, JointConnection, MovingSplineDot, Segment, SegmentConnection};
use alloc::vec;
use alloc::vec::Vec;
use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;
use bevy_math::Dir2;
use bevy_playdate::input::{CrankInput, InputPlugin};
use bevy_playdate::sprite::Sprite;
use bevy_playdate::time::{Time, TimePlugin};
use core::f32::consts::{FRAC_PI_2, PI, TAU};
use curve::arc::ArcSegment;
use curve::line::LineSegment;
use curve::traits::{CurveSegment, CurveType};
use glam::Vec2;
use num_traits::float::Float;
use pd::graphics::bitmap::LCDColorConst;
use pd::graphics::color::Color;
use pd::graphics::Graphics;
use pd::sys::ffi::LCDColor;
use playdate::system::System as PDSystem;
use rand::SeedableRng;
use smallvec::smallvec;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, ScheduleLabel)]
pub struct AppUpdate;

pub fn register_systems(app: &mut App) {
    // graphics.set_
    app
        .add_plugins(InputPlugin)
        .add_plugins(TimePlugin);
    
    app.insert_non_send_resource(Graphics::Cached());
    
    app.add_systems(Update, (
        move_spline_dot
    ).chain());
    
    let t = PDSystem::Default().seconds_since_epoch();
    let mut r = rand_pcg::Pcg32::seed_from_u64(t as u64);

    // let points = vec![
    //     Vec2::new(5.0, 233.0),
    //     Vec2::new(268.0, 233.0),
    //     Vec2::new(183.0, 185.0),
    //     Vec2::new(412.0, 125.0),
    //     Vec2::new(26.0, 95.0),
    //     Vec2::new(270.0, 59.0),
    // ];
    // 
    // let spline = BSpline::new(points, true);
    // let spline = BCurve::from(spline);
    // 
    // let (mut start, mut end) = spline.bounds();
    // start -= Vec2::splat(20.0);
    // end += Vec2::splat(40.0);
    // let width = (end.x - start.x).abs() as i32;
    // let height = (end.y - start.y).abs() as i32;
    // let mut sprite = Sprite::new_from_draw(width, height, Color::CLEAR, |gfx| {
    //     spline.draw(gfx);
    // });
    // 
    // sprite.move_to(230.0, 115.0);
    // 
    // // sprite.move_to()
    // 
    // let spline = app.world_mut().spawn(
    //     (
    //         sprite,
    //         spline,
    //     )
    // ).id();
    // 
    // let mut sprite = Sprite::new_from_draw(10, 10, Color::CLEAR, |gfx| {
    //     gfx.draw_ellipse(0, 0, 10, 10, 4, 0.0, 0.0, LCDColor::BLACK);
    // });
    // 
    // 
    // 
    // sprite.set_center(0.5, 0.5);
    // 
    // app.world_mut().spawn_batch(
    //     (0..1).map(move |i| {
    // 
    //         // println!("{i}");
    //         (
    //             sprite.clone(),
    //             MovingSplineDot {
    //                 t: i as f32 / 10.0,
    //                 v: 0.0,//r.gen_range(-0.005..0.005),
    //                 spline_entity: spline,
    //             }
    //             // MovingDot {
    //             //     x: r.gen_range(0.0..400.0),
    //             //     y: r.gen_range(0.0..240.0),
    //             //     x_v: 0.0, //r.gen_range(-6.0..6.0),
    //             //     y_v: 0.0, //r.gen_range(-6.0..6.0),
    //             // }
    //         )})
    // );
    // 
    // let seg = Segment {
    //     curve: CurveType::Arc(ArcSegment {
    //         center: Vec2::new(20.0, 100.0),
    //         start: 0.0,
    //         end: 6.0,
    //         radius: 20.0,
    //     }),
    //     parent: Entity::PLACEHOLDER,
    //     start_joint: Entity::PLACEHOLDER,
    //     end_joint: Entity::PLACEHOLDER,
    // };
    // 
    // let mut sprite = seg.to_sprite(Graphics::Cached(), 4, LCDColor::BLACK);
    // sprite.move_to(200.0, 150.0);
    // 
    // app.world_mut().spawn((
    //     seg,
    //     sprite,
    // ));
    
    // CurveBuilder::new(Vec2::new(100.0, 200.0), Vec2::NEG_Y)
    //     .push(line(100.0))
    //     .push(arc_curvature_length(50.0, -1.0 / 25.0))
    //     .push(arc_curvature_length(50.0, 1.0 / 25.0))
    //     .push(arc_curvature_length(200.0, -1.0 / 50.0))
    //     .build(&mut app.world_mut().commands(), 10);

    let (segments, joints) = CurveBuilder::new(Vec2::new(168.0, 20.0), Vec2::X)
        .push(line(100.0))
        .push(arc(50.0, -0.25))
        .push(line(50.0))
        .push(arc(25.0, -0.5))
        .push(arc(25.0, 0.5))
        .push(arc(100.0, -0.75))
        .push(line(50.0))
        .build(&mut app.world_mut().commands(), 4, true);



    let mut sprite = Sprite::new_from_draw(10, 10, Color::CLEAR, |gfx| {
        gfx.draw_ellipse(0, 0, 10, 10, 4, 0.0, 0.0, LCDColor::BLACK);
    });
    
    app.world_mut().commands().spawn_batch(
        (0..3).into_iter()
            .map(move |i| (
                sprite.clone(),
                MovingSplineDot {
                    t: i as f32 * 0.1,
                    v: 0.5,
                    spline_entity: segments[i],
                },
            )
            )
    );
    
    
    test_branch(&mut app.world_mut().commands());
    // test_3_way_curve(&mut app.world_mut().commands());
    test_circle(&mut app.world_mut().commands());
    
    // let test = Joint2::new
    
    
    // app.world_mut().spawn(())
    
    // schedule.run(world);
}

fn test_branch(commands: &mut Commands) {
    // testing >- fork shape
    // let mut test_world = World::new();
    // 0 -> top, 1 -> bottom, 2 -> right
    let mut segments = Vec::with_capacity(3);
    for _ in 0..3 {
        segments.push(commands.spawn_empty().id());
    }
    // 0 -> top start
    // 1 -> bottom start
    // 2 -> right end
    // 3 -> middle joint
    let mut joints = Vec::with_capacity(4);
    for _ in 0..4 {
        joints.push(commands.spawn_empty().id());
    }
    
    // top
    commands.entity(segments[0])
        .insert(Segment {
            curve: CurveType::Line(LineSegment {
                start: Vec2::new(20.0, 20.0),
                end: Vec2::new(100.0, 100.0),
            }),
            parent: Entity::PLACEHOLDER,
            start_joint: joints[0],
            end_joint: joints[3],
        }.to_bundle(4));
    
    // bottom
    commands.entity(segments[1])
        .insert(Segment {
            curve: CurveType::Line(LineSegment {
                start: Vec2::new(20.0, 180.0),
                end: Vec2::new(100.0, 100.0),
            }),
            parent: Entity::PLACEHOLDER,
            start_joint: joints[1],
            end_joint: joints[3],
        }.to_bundle(4));
    
    // right
    commands.entity(segments[2])
        .insert(Segment {
            curve: CurveType::Line(LineSegment {
                start: Vec2::new(100.0, 100.0),
                end: Vec2::new(200.0, 100.0),
            }),
            parent: Entity::PLACEHOLDER,
            start_joint: joints[3],
            end_joint: joints[2],
        }.to_bundle(4));
    
    commands.entity(joints[0])
        .insert(Joint2 {
            connections: vec![
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: segments[0],
                            t: 0.0,
                        }
                    ],
                }
            ],
        });

    commands.entity(joints[1])
        .insert(Joint2 {
            connections: vec![
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: segments[1],
                            t: 0.0,
                        }
                    ],
                }
            ],
        });

    commands.entity(joints[2])
        .insert(Joint2 {
            connections: vec![
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: segments[2],
                            t: 1.0,
                        }
                    ],
                }
            ],
        });
    
    commands.entity(joints[3])
        .insert(Joint2 {
            connections: vec![
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: segments[0],
                            t: 1.0, 
                        }
                    ],
                },
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: segments[1],
                            t: 1.0, 
                        }
                    ],
                },
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: segments[2],
                            t: 0.0, 
                        }
                    ],
                },
            ],
        });
    
    let mut sprite = Sprite::new_from_draw(10, 10, Color::CLEAR, |gfx| {
        gfx.draw_ellipse(0, 0, 10, 10, 4, 0.0, 0.0, LCDColor::BLACK);
    });
    
    commands.spawn((
        sprite,
        MovingSplineDot {
            t: 0.5,
            v: 0.0,
            spline_entity: segments[0],
        },
    ));    
}

fn test_3_way_curve(commands: &mut Commands) {
    let top_segment = commands.spawn_empty().id();
    let left_segment = commands.spawn_empty().id();
    let right_segment = commands.spawn_empty().id();
    let left_top_segment = commands.spawn_empty().id();
    let right_top_segment = commands.spawn_empty().id();
    let left_right_segment = commands.spawn_empty().id();

    let top_single_joint = commands.spawn_empty().id();
    let top_multi_joint = commands.spawn_empty().id();
    let left_single_joint = commands.spawn_empty().id();
    let left_multi_joint = commands.spawn_empty().id();
    let right_single_joint = commands.spawn_empty().id();
    let right_multi_joint = commands.spawn_empty().id();
    
    let top_left = Vec2::new(100.0, 50.0);
    let line_width = 4;
    let scale = 50.0;
    
    commands.entity(top_segment)
        .insert(
            Segment {
                curve: CurveType::Line(LineSegment {
                    start: top_left + Vec2::new(scale * 2.0, 0.0),
                    end: top_left + Vec2::new(scale * 2.0, scale),
                }),
                parent: Entity::PLACEHOLDER,
                start_joint: top_single_joint,
                end_joint: top_multi_joint,
            }.to_bundle(line_width)
        );
    
    commands.entity(left_segment)
        .insert(
            Segment {
                curve: CurveType::Line(LineSegment {
                    start: top_left + Vec2::new(0.0, scale * 2.0),
                    end: top_left + Vec2::new(scale, scale * 2.0),
                }),
                parent: Entity::PLACEHOLDER,
                start_joint: left_single_joint,
                end_joint: left_multi_joint,
            }.to_bundle(line_width)
        );
    
    commands.entity(right_segment)
        .insert(
            Segment {
                curve: CurveType::Line(LineSegment {
                    start: top_left + Vec2::new(scale * 4.0, scale * 2.0),
                    end: top_left + Vec2::new(scale * 3.0, scale * 2.0),
                }),
                parent: Entity::PLACEHOLDER,
                start_joint: right_single_joint,
                end_joint: right_multi_joint,
            }.to_bundle(line_width)
        );
    
    commands.entity(left_top_segment)
        .insert(
            Segment {
                curve: CurveType::Arc(ArcSegment {
                    center: top_left + Vec2::new(scale, scale),
                    start: -FRAC_PI_2,
                    end: 0.0,
                    radius: scale,
                }),
                parent: Entity::PLACEHOLDER,
                start_joint: left_multi_joint,
                end_joint: top_multi_joint,
            }
                .to_bundle(line_width)
        );

    commands.entity(right_top_segment)
        .insert(
            Segment {
                curve: CurveType::Arc(ArcSegment {
                    center: top_left + Vec2::new(scale * 3.0, scale),
                    start: -FRAC_PI_2,
                    end: -PI,
                    radius: scale,
                }),
                parent: Entity::PLACEHOLDER,
                start_joint: right_multi_joint,
                end_joint: top_multi_joint,
            }
                .to_bundle(line_width)
        );
    
    commands.entity(left_right_segment)
        .insert(
            Segment {
                curve: CurveType::Line(LineSegment {
                    start: top_left + Vec2::new(scale, scale * 2.0),
                    end: top_left + Vec2::new(scale * 3.0, scale * 2.0),
                }),
                parent: Entity::PLACEHOLDER,
                start_joint: left_multi_joint,
                end_joint: right_multi_joint,
            }
                .to_bundle(line_width)
        );
    
    commands.entity(top_single_joint)
        .insert(Joint2 {
            connections: vec![
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: top_segment,
                            t: 0.0,
                        }
                    ],
                }
            ],
        });

    commands.entity(left_single_joint)
        .insert(Joint2 {
            connections: vec![
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: left_segment,
                            t: 0.0,
                        }
                    ],
                }
            ],
        });

    commands.entity(right_single_joint)
        .insert(Joint2 {
            connections: vec![
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: right_segment,
                            t: 0.0,
                        }
                    ],
                }
            ],
        });
    
    commands.entity(top_multi_joint)
        .insert(Joint2 {
            connections: vec![
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: top_segment,
                            t: 1.0,
                        }
                    ],
                },
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: left_top_segment,
                            t: 1.0,
                        },
                        SegmentConnection {
                            id: right_top_segment,
                            t: 1.0,
                        },
                    ],
                },
            ]
        });
    
    commands.entity(left_multi_joint)
        .insert(Joint2 {
            connections: vec![
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: left_segment,
                            t: 1.0,
                        }
                    ],
                },
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: left_right_segment,
                            t: 0.0,
                        },
                        SegmentConnection {
                            id: left_top_segment,
                            t: 0.0,
                        },
                    ],
                },
            ],
        });

    commands.entity(right_multi_joint)
        .insert(Joint2 {
            connections: vec![
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: right_segment,
                            t: 1.0,
                        }
                    ],
                },
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: right_top_segment,
                            t: 0.0,
                        },
                        SegmentConnection {
                            id: left_right_segment,
                            t: 1.0,
                        },
                    ],
                },
            ],
        });
    
    let mut sprite = Sprite::new_from_draw(10, 10, Color::CLEAR, |gfx| {
        gfx.draw_ellipse(0, 0, 10, 10, 4, 0.0, 0.0, LCDColor::BLACK);
    });
    
    commands.spawn_batch(
        (0..10).into_iter()
            .map(move |i| (
                sprite.clone(),
                MovingSplineDot {
                    t: i as f32 * 0.1,
                    v: 0.0,
                    spline_entity: left_segment,
                },
            )
        )
    );
}

fn test_circle(commands: &mut Commands) {
    let circle_segment = commands.spawn_empty().id();
    let joint = commands.spawn_empty().id();
    
    let center = Vec2::new(275.0, 100.0);
    let radius = 75.0;
    
    commands.entity(circle_segment)
        .insert(Segment {
            curve: CurveType::Arc(ArcSegment {
                center,
                start: 0.0,
                end: TAU,
                radius,
            }),
            parent: Entity::PLACEHOLDER,
            start_joint: joint,
            end_joint: joint,
        }.to_bundle(4));
    
    commands.entity(joint)
        .insert(Joint2 {
            connections: vec![
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: circle_segment,
                            t: 0.0,
                        }
                    ],
                },
                JointConnection {
                    segments: smallvec![
                        SegmentConnection {
                            id: circle_segment,
                            t: 1.0,
                        }
                    ],
                },
            ],
        });

    let mut sprite = Sprite::new_from_draw(10, 10, Color::CLEAR, |gfx| {
        gfx.draw_ellipse(0, 0, 10, 10, 4, 0.0, 0.0, LCDColor::BLACK);
    });

    commands.spawn_batch(
        (0..10).into_iter()
            .map(move |i| (
                sprite.clone(),
                MovingSplineDot {
                    t: i as f32 * 0.1,
                    v: 0.0,
                    spline_entity: circle_segment,
                },
            )
            )
    );
}


fn rotate(v: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = (f32::sin(angle), f32::cos(angle));
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

fn move_spline_dot(
    mut dots: Query<(&mut MovingSplineDot, &mut Sprite)>,
    q_segments: Query<&Segment>,
    q_joints: Query<&Joint2>,
    crank: Res<CrankInput>,
    time: Res<Time>,
) {
    let gravity = rotate(Vec2::NEG_Y, crank.angle.to_radians()) * 100.0;
    
    for (mut dot, mut sprite) in &mut dots {
        // dot.t = (dot.t + dot.v * 0.02) % 1.0;
        let segment = q_segments.get(dot.spline_entity).unwrap();
        
        let dir = segment.curve.dir(dot.t);
        // TODO: make frame-rate independent
        dot.v += 0.5 * gravity.dot(dir.into()) * time.delta_seconds();
        dot.t += dot.v * time.delta_seconds() / segment.curve.length();
        dot.v += 0.5 * gravity.dot(dir.into()) * time.delta_seconds();
        
        if dot.t > 1.0 || dot.t < 0.0 {
            let (j, t) = if dot.t > 1.0 {
                (segment.end_joint, 1.0)
            } else {
                (segment.start_joint, 0.0)
            };
            
            let joint = q_joints.get(j).unwrap();
            let result = joint.enter(
                dot.v,
                Dir2::new(gravity).unwrap(),
                dot.spline_entity,
                t,
                &q_segments,
            );

            dot.t = result.t;
            dot.v = result.v;
            dot.spline_entity = result.next;
        }
        
        let new_pos = q_segments.get(dot.spline_entity).unwrap()
            .curve.position(dot.t);
        
        sprite.move_to(new_pos.x, new_pos.y);
    }
}

