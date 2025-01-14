use crate::sprite::Sprite;
use bevy_app::{App, Plugin, PostUpdate};
use bevy_ecs::change_detection::*;
use bevy_ecs::prelude::*;
use bevy_math::{Affine2, Affine3A};
use bevy_transform::prelude::{GlobalTransform, Transform};
use core::ops::Deref;

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            view_system
                .after(bevy_transform::systems::propagate_transforms)
                .after(bevy_transform::systems::sync_simple_transforms)
                .before(Sprite::draw_sprites),
        );
    }
}

/// Add this marker component to an entity to set it as the
#[derive(Component, Copy, Clone, Eq, PartialEq)]
#[require(Transform)]
pub struct Camera;

#[derive(Copy, Clone, PartialEq, Component)]
pub struct CameraView(pub Affine2);

// Either camera has moved
// or single object moved
pub fn view_system(
    camera: Option<Single<Ref<GlobalTransform>, With<Camera>>>,
    mut q_sprites: Query<(Ref<GlobalTransform>, &mut Sprite)>,
) {
    if let Some(camera_transform) = camera {
        let inv = camera_transform.deref().affine().inverse();
        if camera_transform.deref().is_changed() {
            for (transform, mut spr) in q_sprites.iter_mut() {
                let relative = inv * transform.deref().affine();
                set_sprite_affine(spr.as_mut(), relative);
            }
        } else {
            for (transform, mut spr) in q_sprites.iter_mut() {
                if !transform.is_changed() {
                    continue;
                }

                let relative = inv * transform.deref().affine();
                set_sprite_affine(spr.as_mut(), relative);
            }
        }
    } else {
        for (transform, mut spr) in q_sprites.iter_mut() {
            if !transform.is_changed() {
                continue;
            }

            set_sprite_affine(spr.as_mut(), transform.affine());
        }
    }
}

pub fn set_sprite_affine(sprite: &mut Sprite, affine: Affine3A) {
    let (scale, rot, trans) = affine.to_scale_rotation_translation();
    // let two_d = Affine2::from_scale_angle_translation(scale.into(), rot.);

    sprite.move_to(trans.x, trans.y);
}
