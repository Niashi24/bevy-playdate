use alloc::{format, vec};
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::ffi::c_int;
use core::mem::swap;
use core::ops::Deref;
use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;
use bevy_ecs::system::RunSystemOnce;
use glam::{FloatExt, Vec2};
use pd::graphics::api::{Api, Cache};
use pd::graphics::color::{Color, LCDColorConst};
use pd::graphics::{BitmapFlip, Graphics};
use pd::sys::ffi::LCDColor;
use pd::sys::traits::AsRaw;
use pd::system::prelude::PDDateTime;
use playdate::system::System as PDSystem;
use rand::{Rng, SeedableRng};
use bevy_playdate::input::{AccelerometerInput, CrankInput, InputPlugin};
use bevy_playdate::sprite::Sprite;
use curve::BSpline;
use crate::curve::{BCurve, BCurveFallSystem};
use num_traits::float::Float;
use num_traits::{Euclid, FloatConst};
use pd::graphics::bitmap::Bitmap;
use bevy_playdate::dbg;
use bevy_playdate::time::{Time, TimePlugin};
use curve::arc::ArcSegment;
use curve::line::LineSegment;
use curve::traits::{CurveSegment, CurveType};
use crate::builder::{CurveBuilder, MovingSplineDot, Segment};
use crate::builder::builders::{arc_curvature_length, arc, line};

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

    CurveBuilder::new(Vec2::new(168.0, 20.0), Vec2::X)
        .push(line(100.0))
        .push(arc(50.0, -0.25))
        .push(line(50.0))
        .push(arc(25.0, -0.5))
        .push(arc(25.0, 0.5))
        .push(arc(100.0, -0.75))
        .push(line(50.0))
        .build(&mut app.world_mut().commands(), 1);
    
    
    // app.world_mut().spawn(())
    
    // schedule.run(world);
}

fn rotate(v: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = (f32::sin(angle), f32::cos(angle));
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

fn move_spline_dot(
    mut dots: Query<(&mut MovingSplineDot, &mut Sprite)>,
    splines: Query<&BCurve>,
    crank: Res<CrankInput>,
    time: Res<Time>,
) {
    let gravity = rotate(Vec2::NEG_Y, crank.angle.to_radians()) * 50.0;
    
    for (mut dot, mut sprite) in &mut dots {
        // dot.t = (dot.t + dot.v * 0.02) % 1.0;
        let spline = splines.get(dot.spline_entity).unwrap();
        
        let old_pos = spline.position(dot.t);
        let old_t = dot.t;
        
        (dot.t, dot.v) = BCurveFallSystem::new(spline, gravity)
            .integrate(dot.t, dot.v, time.delta_seconds());
        
        dot.t = f32::rem_euclid(&dot.t, &1.0f32);
        
        let new_pos = spline.position(dot.t);
        // println!("Expected: {:.3} Actual: {:.3} dp: {:.3}", dot.v.abs(), (new_pos - old_pos).length() / 0.02, (dot.t - old_t) / 0.02);
        dot.v *= 0.995;
        
        sprite.move_to(new_pos.x, new_pos.y);
    }
}

