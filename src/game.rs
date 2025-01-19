use crate::builder::{
    Joint, MovingSplineDot, Segment,
};
use alloc::format;
use bevy_app::{App, PostUpdate, Update};
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;
use bevy_math::Dir2;
use bevy_playdate::input::CrankInput;
use bevy_playdate::sprite::Sprite;
use bevy_playdate::time::Time;
use bevy_transform::prelude::Transform;
use curve::roots::{quadratic, SolutionIter};
use curve::traits::CurveSegment;
use glam::Vec2;
use num_traits::float::{Float, TotalOrder};
use pd::graphics::bitmap::LCDColorConst;
use pd::graphics::text::draw_text;
use pd::graphics::{draw_ellipse, draw_rect, Graphics};
use pd::sprite::draw_sprites;
use pd::sys::ffi::LCDColor;
use playdate::system::System as PDSystem;
use rand::SeedableRng;
use bevy_playdate::debug::in_debug;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, ScheduleLabel)]
pub struct AppUpdate;

pub fn register_systems(app: &mut App) {
    // graphics.set_

    app.insert_non_send_resource(Graphics::Cached());

    app.add_systems(Update, (
        move_spline_dot,
    ).chain());

    app.add_systems(PostUpdate, (debug_dots, debug_sprite_bounds)
        .after(draw_sprites)
        .run_if(in_debug));

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

    crate::test_scenes::test_builder(&mut app.world_mut().commands());
    crate::test_scenes::test_branch(&mut app.world_mut().commands());
    crate::test_scenes::test_3_way_curve(&mut app.world_mut().commands());
    crate::test_scenes::test_circle(&mut app.world_mut().commands());

    // let test = Joint2::new

    // app.world_mut().spawn(())

    // schedule.run(world);
}

fn debug_sprite_bounds(
    sprites: Query<&Sprite>,
) {
    for sprite in sprites.iter() {
        let bounds = sprite.bounds();
        draw_rect(bounds.x as i32, bounds.y as i32, bounds.width as i32, bounds.height as i32, LCDColor::BLACK);
        let (x, y) = sprite.position();
        draw_ellipse(x as i32 - 2, y as i32 - 2, 4, 4, 2, 0.0, 360.0, LCDColor::XOR);
        // draw_rect(x as i32, y as i32, 5, 5, LCDColor::XOR);
    }
}

fn rotate(v: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = (f32::sin(angle), f32::cos(angle));
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

fn move_spline_dot(
    mut dots: Query<(&mut MovingSplineDot, &mut Transform)>,
    q_segments: Query<&Segment>,
    q_joints: Query<&Joint>,
    crank: Res<CrankInput>,
    time: Res<Time>,
) {
    let gravity = rotate(Vec2::NEG_Y, crank.angle.to_radians()) * 100.0;

    for (mut dot, mut transform) in &mut dots {
        move_dot_recursive(
            dot.as_mut(),
            time.delta_seconds(),
            0,
            gravity,
            &q_segments,
            &q_joints,
        );
        dot.v *= 0.999;

        let new_pos = q_segments
            .get(dot.spline_entity)
            .unwrap()
            .curve
            .position(dot.t);

        transform.translation = new_pos.extend(0.0);

        // sprite.move_to(new_pos.x, new_pos.y);
    }
}

// todo:
//  velocity direction should be kept through iterations
//  so it is the same w/ or w/o the joint
//  i.e. acceleration added only once
fn move_dot_recursive(
    dot: &mut MovingSplineDot,
    t_remaining: f32,
    depth: usize,
    gravity: Vec2,
    q_segments: &Query<&Segment>,
    q_joints: &Query<&Joint>,
) {
    if depth > 10 {
        println!("spent too long: remaining time: {}", t_remaining);
        return;
    }
    if t_remaining < 1e-6 {
        return;
    }

    let segment = q_segments.get(dot.spline_entity).unwrap();
    let length = segment.curve.length();

    let dir = segment.curve.dir(dot.t);
    let g = gravity.dot(dir.into());
    if g == 0.0 && dot.v == 0.0 {
        return;
    }

    let v = dot.v;
    let t = dot.t;

    fn change_joints(
        dot: &mut MovingSplineDot,
        new_joint: Entity,
        old_t: f32,
        gravity: Vec2,
        q_segments: &Query<&Segment>,
        q_joints: &Query<&Joint>,
    ) {
        let joint = q_joints.get(new_joint).unwrap();
        let result = joint.enter(
            dot.v,
            Dir2::new(gravity).unwrap(),
            dot.spline_entity,
            old_t,
            &q_segments,
        );

        dot.t = result.t;
        dot.v = result.v;
        dot.spline_entity = result.next;
    }

    // this is really just for the case where joint is not connected (dot is clamped and stopped),
    // then is reconnected, which might be a mechanic but otherwise probably won't happen
    if (t == 1.0 && (v > 0.0 || (v == 0.0 && g > 0.0))) {
        let old_dot = *dot;
        change_joints(dot, segment.end_joint, t, gravity, q_segments, q_joints);
        if *dot == old_dot {
            return;
        }

        return move_dot_recursive(dot, t_remaining, depth + 1, gravity, q_segments, q_joints);
    }
    if (t == 0.0 && (v < 0.0 || (v == 0.0 && g < 0.0))) {
        let old_dot = *dot;
        change_joints(dot, segment.start_joint, t, gravity, q_segments, q_joints);
        if *dot == old_dot {
            return;
        }

        return move_dot_recursive(dot, t_remaining, depth + 1, gravity, q_segments, q_joints);
    }

    // solve t = 1
    // => 1/2 * g * t^2 + v * t + t_0 = 1
    // => ... - 1 = 0
    // where t > 0 and t < remaining
    let end = SolutionIter::from(quadratic(0.5 * g / length, dot.v / length, dot.t - 1.0))
        .filter(|&t| t > 0.0 && t <= t_remaining)
        .map(|t| (t, 1.0f32, segment.end_joint))
        .chain(
            SolutionIter::from(quadratic(0.5 * g / length, dot.v / length, dot.t))
                .filter(|&t| t > 0.0 && t <= t_remaining)
                .map(|t| (t, 0.0f32, segment.start_joint)),
        )
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .into_iter()
        .next();

    if let Some((time, t_old, joint)) = end {
        dot.v += g * time / length;

        let old_dot = *dot;
        change_joints(dot, joint, t_old, gravity, q_segments, q_joints);
        if *dot == old_dot {
            return;
        }

        return move_dot_recursive(
            dot,
            t_remaining - time,
            depth + 1,
            gravity,
            q_segments,
            q_joints,
        );
    }

    dot.t += (0.5 * g * t_remaining.powi(2) + dot.v * t_remaining) / length;
    dot.v += g * t_remaining;

    dot.t = dot.t.clamp(0.0, 1.0);
}

// todo:
//  this approach could work but if the ball enters and exits a joint (or multiple)
//  in the same frame it won't trigger (what if we want effects in the future?)
//      ok maybe it doesn't matter
fn move_dot_2(
    dot: &mut MovingSplineDot,
    gravity: Vec2,
    q_segments: &Query<&Segment>,
    q_joints: &Query<&Joint>,
    time: f32,
) {
    let segment = q_segments.get(dot.spline_entity).unwrap();
    let length = segment.curve.length();

    let dir = segment.curve.dir(dot.t);
    let g = gravity.dot(dir.into());

    if dot.v == 0.0 && g == 0.0 {
        return;
    }

    let mut traveled = (0.5 * g * time + dot.v) * time;

    dot.t += traveled / length;
    dot.v += g * time;

    while dot.t > 1.0 {
        dot.t -= 1.0;

        let joint = q_joints.get(segment.end_joint).unwrap();
        let result = joint.enter(
            dot.v,
            Dir2::new(gravity).unwrap(),
            dot.spline_entity,
            1.0,
            &q_segments,
        );

        dot.t = result.t;
        dot.v = result.v;
        dot.spline_entity = result.next;
    }
}

// fn move_recursive_2(
//     remaining_distance: f32,
// ) {
//     let distance_to_traverse = f(t, v, length);
//
//     if remaining_distance >= distance_to_traverse {
//
//     }
// }

fn debug_dots(q_dots: Query<(&MovingSplineDot, &Sprite)>) {
    for (dot, spr) in q_dots.iter() {
        let (x, y) = spr.position();
        draw_text(
            &format!("t: {:.2?}, v: {:.2?}", dot.t, dot.v),
            x as i32 - 55,
            y as i32 + 10,
        )
        .unwrap();
    }
}
