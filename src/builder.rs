use crate::curve::{Joint, JointConnection, Segment, SegmentConnection};
use alloc::vec::Vec;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Commands};
use bevy_ecs::query::{QueryData, QueryFilter};
use core::fmt::Debug;
use bevy_ecs::name::Name;
use bevy_transform::prelude::Transform;
use curve::arc::ArcSegment;
use curve::line::LineSegment;
use curve::traits::{CurveSegment, CurveType};
use glam::{FloatExt, Vec2};
use num_traits::Euclid;
use pd::graphics::api::Api;
use pd::graphics::bitmap::LCDColorConst;
use smallvec::smallvec;

pub struct CurveBuilder {
    segments: Vec<SplineSegment>,
    cur_pos: Vec2,
    cur_dir: Vec2,
    total_length: f32,
}

impl CurveBuilder {
    pub fn new(pos: Vec2, dir: Vec2) -> Self {
        CurveBuilder {
            segments: Vec::new(),
            cur_pos: pos,
            cur_dir: dir,
            total_length: 0.0,
        }
    }

    pub fn push(mut self, builder: impl SectionBuilder) -> Self {
        let segment = builder.add_segment(&mut self.cur_pos, &mut self.cur_dir);

        let len = segment.length();

        self.segments.push(SplineSegment {
            running_distance: self.total_length,
            segment,
        });

        self.total_length += len;

        self
    }

    /// Adds a new segment to the curve with a `length` and `curvature`.
    ///
    /// `curvature == 0.0` is a straight line.
    /// `curvature > 0.0` is a curved arc to the left.
    /// `curvature < 0.0` is a curved arc to the right.
    /// Higher `abs(curvature)` the sharper it turns.
    pub fn segment(mut self, length: f32, curvature: f32) -> Self {
        if length == 0.0 {
            return self;
        }

        let segment: CurveType = if curvature == 0.0 {
            let start = self.cur_pos;
            let end = self.cur_pos + self.cur_dir * length;
            self.cur_pos = end;

            LineSegment { start, end }.into()
        } else {
            let arc = ArcSegment::from_pos_dir_curvature_length(
                self.cur_pos,
                self.cur_dir,
                curvature,
                length,
            );

            self.cur_pos = arc.position(1.0);
            self.cur_dir = arc.dir(1.0).normalize_or_zero();

            arc.into()
        };

        println!("{:?}", segment);

        println!(
            "start: {:.2?} w/ {:.2?}, end: {:.2?} w/ {:.2?}",
            segment.position(0.0),
            segment.dir(0.0).normalize_or_zero(),
            segment.position(1.0),
            segment.dir(1.0).normalize_or_zero()
        );

        self.segments.push(SplineSegment {
            running_distance: self.total_length,
            segment,
        });

        self.total_length += length;

        self
    }

    pub fn build(
        self,
        commands: &mut Commands,
        line_width: i32,
        join_ends: bool,
    ) -> (Vec<Entity>, Vec<Entity>) {
        if self.segments.is_empty() {
            return Default::default();
        }
        // TODO when bevy_hierarchy ports, make children
        let parent = commands.spawn(Transform::default()).id();

        let seg_entities: Vec<Entity> = (0..self.segments.len())
            .map(|_| commands.spawn(Name::new("Segment")).id())
            .collect();
        let joint_entities: Vec<Entity> = (0..self.segments.len() + if join_ends { 0 } else { 1 })
            .map(|_| commands.spawn(Name::new("Joint")).id())
            .collect();
        
        commands.entity(parent).add_children(&seg_entities).add_children(&joint_entities);

        // Spawn joints

        for i in 1..self.segments.len() {
            commands.entity(joint_entities[i]).insert(Joint {
                connections: smallvec![
                    JointConnection {
                        segments: smallvec![SegmentConnection {
                            id: seg_entities[i - 1],
                            t: 1.0,
                        }],
                    },
                    JointConnection {
                        segments: smallvec![SegmentConnection {
                            id: seg_entities[i],
                            t: 0.0,
                        }],
                    },
                ],
            });
        }

        if join_ends {
            commands.entity(joint_entities[0]).insert(Joint {
                connections: smallvec![
                    JointConnection {
                        segments: smallvec![SegmentConnection {
                            id: seg_entities[0],
                            t: 0.0,
                        },],
                    },
                    JointConnection {
                        segments: smallvec![SegmentConnection {
                            id: *seg_entities.last().unwrap(),
                            t: 1.0,
                        },],
                    },
                ],
            });
        } else {
            commands.entity(joint_entities[0]).insert(Joint {
                connections: smallvec![JointConnection {
                    segments: smallvec![SegmentConnection {
                        id: seg_entities[0],
                        t: 0.0,
                    }],
                }],
            });

            commands
                .entity(*joint_entities.last().unwrap())
                .insert(Joint {
                    connections: smallvec![JointConnection {
                        segments: smallvec![SegmentConnection {
                            id: *seg_entities.last().unwrap(),
                            t: 1.0,
                        },],
                    }],
                });
        }

        // Spawn segments
        let mut segments = self.segments.into_iter();

        for i in 0..seg_entities.len() {
            let segment = Segment {
                curve: segments.next().unwrap().segment,
                start_joint: joint_entities[i],
                end_joint: joint_entities[(i + 1) % joint_entities.len()],
            };

            commands
                .entity(seg_entities[i])
                .insert(segment.to_bundle(line_width));
        }

        (seg_entities, joint_entities)
    }
}

pub trait SectionBuilder {
    /// takes in previous
    fn add_segment(self, pos: &mut Vec2, dir: &mut Vec2) -> CurveType;
}

impl<F: FnOnce(&mut Vec2, &mut Vec2) -> CurveType> SectionBuilder for F {
    fn add_segment(self, pos: &mut Vec2, dir: &mut Vec2) -> CurveType {
        self(pos, dir)
    }
}

pub mod builders {
    use crate::builder::SectionBuilder;
    use curve::arc::ArcSegment;
    use curve::line::LineSegment;
    use curve::traits::CurveSegment;
    use glam::Vec2;
    use num_traits::FloatConst;

    pub fn line(length: f32) -> impl SectionBuilder {
        move |pos: &mut Vec2, dir: &mut Vec2| {
            let start = *pos;
            let end = *pos + *dir * length;
            *pos = end;

            LineSegment { start, end }.into()
        }
    }

    pub fn arc_curvature_length(length: f32, curvature: f32) -> impl SectionBuilder {
        move |pos: &mut Vec2, dir: &mut Vec2| {
            let arc = ArcSegment::from_pos_dir_curvature_length(*pos, *dir, curvature, length);

            *pos = arc.position(1.0);
            *dir = arc.dir(1.0).normalize_or_zero();

            arc.into()
        }
    }

    /// Appends an arc with the given radius and number of revolutions (1 revolution = 1 full circle).
    /// Use positive revolutions to go counterclockwise, negative for clockwise.
    /// Panics if you put in a negative radius.
    pub fn arc(radius: f32, revolutions: f32) -> impl SectionBuilder {
        let curvature = revolutions.signum() / radius;
        let length = radius * revolutions.abs() * f32::TAU();
        arc_curvature_length(length, curvature)
    }
}

struct SplineSegment {
    pub running_distance: f32,
    pub segment: CurveType,
}
