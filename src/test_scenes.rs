use alloc::vec;
use alloc::vec::Vec;
use core::cell::LazyCell;
use core::f32::consts::{FRAC_PI_2, PI, TAU};
use bevy_ecs::entity::Entity;
use bevy_ecs::name::Name;
use bevy_ecs::prelude::Commands;
use bevy_transform::components::Transform;
use glam::Vec2;
use pd::graphics::bitmap::Color;
use pd::graphics::color::LCDColorConst;
use pd::sys::ffi::LCDColor;
use smallvec::smallvec;
use bevy_playdate::sprite::Sprite;
use curve::arc::ArcSegment;
use curve::line::LineSegment;
use curve::traits::CurveType;
use crate::builder::builders::{arc, line};
use crate::builder::{CurveBuilder, Joint, JointConnection, MovingSplineDot, Segment, SegmentConnection};

pub fn test_builder(commands: &mut Commands) {
    let (segments, joints) = CurveBuilder::new(Vec2::new(168.0, 20.0), Vec2::X)
        .push(line(100.0))
        .push(arc(50.0, -0.25))
        .push(line(50.0))
        .push(arc(25.0, -0.5))
        .push(arc(25.0, 0.5))
        .push(arc(100.0, -0.75))
        .push(line(50.0))
        .build(commands, 4, true);

    let mut sprite = Sprite::new_from_draw(10, 10, Color::CLEAR, |gfx| {
        gfx.draw_ellipse(0, 0, 10, 10, 4, 0.0, 0.0, LCDColor::BLACK);
    });

    commands.spawn_batch((0..1).into_iter().map(move |i| {
        (
            sprite.clone(),
            MovingSplineDot {
                t: i as f32 * 0.1,
                v: 0.5,
                spline_entity: segments[i],
            },
        )
    }));
}

pub fn test_branch(commands: &mut Commands) {
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
    commands.entity(segments[0]).insert(
        Segment {
            curve: CurveType::Line(LineSegment {
                start: Vec2::new(20.0, 20.0),
                end: Vec2::new(100.0, 100.0),
            }),
            parent: Entity::PLACEHOLDER,
            start_joint: joints[0],
            end_joint: joints[3],
        }
            .to_bundle(4),
    );

    // bottom
    commands.entity(segments[1]).insert(
        Segment {
            curve: CurveType::Line(LineSegment {
                start: Vec2::new(20.0, 180.0),
                end: Vec2::new(100.0, 100.0),
            }),
            parent: Entity::PLACEHOLDER,
            start_joint: joints[1],
            end_joint: joints[3],
        }
            .to_bundle(4),
    );

    // right
    commands.entity(segments[2]).insert(
        Segment {
            curve: CurveType::Line(LineSegment {
                start: Vec2::new(100.0, 100.0),
                end: Vec2::new(200.0, 100.0),
            }),
            parent: Entity::PLACEHOLDER,
            start_joint: joints[3],
            end_joint: joints[2],
        }
            .to_bundle(4),
    );

    commands.entity(joints[0]).insert(Joint {
        connections: vec![JointConnection {
            segments: smallvec![SegmentConnection {
                id: segments[0],
                t: 0.0,
            }],
        }],
    });

    commands.entity(joints[1]).insert(Joint {
        connections: vec![JointConnection {
            segments: smallvec![SegmentConnection {
                id: segments[1],
                t: 0.0,
            }],
        }],
    });

    commands.entity(joints[2]).insert(Joint {
        connections: vec![JointConnection {
            segments: smallvec![SegmentConnection {
                id: segments[2],
                t: 1.0,
            }],
        }],
    });

    commands.entity(joints[3]).insert(Joint {
        connections: vec![
            JointConnection {
                segments: smallvec![SegmentConnection {
                    id: segments[0],
                    t: 1.0,
                }],
            },
            JointConnection {
                segments: smallvec![SegmentConnection {
                    id: segments[1],
                    t: 1.0,
                }],
            },
            JointConnection {
                segments: smallvec![SegmentConnection {
                    id: segments[2],
                    t: 0.0,
                }],
            },
        ],
    });

    let mut sprite = Sprite::new_from_draw(10, 10, Color::CLEAR, |gfx| {
        gfx.draw_ellipse(0, 0, 10, 10, 4, 0.0, 0.0, LCDColor::BLACK);
    });

    commands.spawn((
        sprite,
        Transform::default(),
        MovingSplineDot {
            t: 0.5,
            v: 0.0,
            spline_entity: segments[0],
        },
    ));
}

pub fn test_3_way_curve(commands: &mut Commands) {
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
    let line_width = 3;
    let scale = 50.0;

    commands.entity(top_segment).insert(
        Segment {
            curve: CurveType::Line(LineSegment {
                start: top_left + Vec2::new(scale * 2.0, 0.0),
                end: top_left + Vec2::new(scale * 2.0, scale),
            }),
            parent: Entity::PLACEHOLDER,
            start_joint: top_single_joint,
            end_joint: top_multi_joint,
        }
            .to_bundle(line_width),
    );

    commands.entity(left_segment).insert(
        Segment {
            curve: CurveType::Line(LineSegment {
                start: top_left + Vec2::new(0.0, scale * 2.0),
                end: top_left + Vec2::new(scale, scale * 2.0),
            }),
            parent: Entity::PLACEHOLDER,
            start_joint: left_single_joint,
            end_joint: left_multi_joint,
        }
            .to_bundle(line_width),
    );

    commands.entity(right_segment).insert(
        Segment {
            curve: CurveType::Line(LineSegment {
                start: top_left + Vec2::new(scale * 4.0, scale * 2.0),
                end: top_left + Vec2::new(scale * 3.0, scale * 2.0),
            }),
            parent: Entity::PLACEHOLDER,
            start_joint: right_single_joint,
            end_joint: right_multi_joint,
        }
            .to_bundle(line_width),
    );

    commands.entity(left_top_segment).insert(
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
            .to_bundle(line_width),
    );

    commands.entity(right_top_segment).insert(
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
            .to_bundle(line_width),
    );

    commands.entity(left_right_segment).insert(
        Segment {
            curve: CurveType::Line(LineSegment {
                start: top_left + Vec2::new(scale, scale * 2.0),
                end: top_left + Vec2::new(scale * 3.0, scale * 2.0),
            }),
            parent: Entity::PLACEHOLDER,
            start_joint: left_multi_joint,
            end_joint: right_multi_joint,
        }
            .to_bundle(line_width),
    );

    commands.entity(top_single_joint).insert(Joint {
        connections: vec![JointConnection {
            segments: smallvec![SegmentConnection {
                id: top_segment,
                t: 0.0,
            }],
        }],
    });

    commands.entity(left_single_joint).insert(Joint {
        connections: vec![JointConnection {
            segments: smallvec![SegmentConnection {
                id: left_segment,
                t: 0.0,
            }],
        }],
    });

    commands.entity(right_single_joint).insert(Joint {
        connections: vec![JointConnection {
            segments: smallvec![SegmentConnection {
                id: right_segment,
                t: 0.0,
            }],
        }],
    });

    commands.entity(top_multi_joint).insert(Joint {
        connections: vec![
            JointConnection {
                segments: smallvec![SegmentConnection {
                    id: top_segment,
                    t: 1.0,
                }],
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
        ],
    });

    commands.entity(left_multi_joint).insert(Joint {
        connections: vec![
            JointConnection {
                segments: smallvec![SegmentConnection {
                    id: left_segment,
                    t: 1.0,
                }],
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

    commands.entity(right_multi_joint).insert(Joint {
        connections: vec![
            JointConnection {
                segments: smallvec![SegmentConnection {
                    id: right_segment,
                    t: 1.0,
                }],
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

    sprite.set_z_index(10);

    commands.spawn_batch((0..10).into_iter().map(move |i| {
        (
            sprite.clone(),
            Transform::default(),
            MovingSplineDot {
                t: i as f32 * 0.1,
                v: 0.0,
                spline_entity: left_segment,
            },
        )
    }));
}

pub fn test_circle(commands: &mut Commands) {
    let circle_segment = commands.spawn_empty().id();
    let joint = commands.spawn_empty().id();

    let center = Vec2::new(200.0, 100.0);
    let radius = 75.0;

    commands
        .entity(circle_segment)
        .insert(
            Segment {
                curve: CurveType::Arc(ArcSegment {
                    center,
                    start: 0.0,
                    end: TAU,
                    radius,
                }),
                parent: Entity::PLACEHOLDER,
                start_joint: joint,
                end_joint: joint,
            }
                .to_bundle(4),
        )
        .insert(Name::new("Circle"));

    commands
        .entity(joint)
        .insert(Joint {
            connections: vec![
                JointConnection {
                    segments: smallvec![SegmentConnection {
                        id: circle_segment,
                        t: 0.0,
                    }],
                },
                JointConnection {
                    segments: smallvec![SegmentConnection {
                        id: circle_segment,
                        t: 1.0,
                    }],
                },
            ],
        })
        .insert(Name::new("Joint"));

    commands.spawn_batch((0..1).into_iter().map(move |i| {
        (
            CIRCLE.clone(),
            Transform::IDENTITY,
            MovingSplineDot {
                t: i as f32 * 0.1,
                v: 0.0,
                spline_entity: circle_segment,
            },
        )
    }));
}

pub const CIRCLE: LazyCell<Sprite> = LazyCell::new(|| {
    Sprite::new_from_draw(10, 10, Color::CLEAR, |gfx| {
        gfx.draw_ellipse(0, 0, 10, 10, 4, 0.0, 0.0, LCDColor::BLACK);
    })
});