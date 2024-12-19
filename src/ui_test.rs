use alloc::rc::Rc;
use alloc::sync::Arc;
use core::ffi::c_int;
use bevy_app::{App, PostUpdate};
use bevy_ecs::prelude::*;
use bevy_playdate::sprite::Sprite;
use pd::graphics::bitmap::{Bitmap, Color};
use pd::sys::ffi::{LCDPattern, LCDSolidColor, PDRect};

pub fn ui_plugin(app: &mut App) {
    app.add_systems(PostUpdate, render_node);
    app.world_mut()
        .spawn((
            ComputedNode {
                rect: PDRect {
                    x: 5.0,
                    y: 40.0,
                    width: 200.0,
                    height: 40.0,
                }
            }
        ));
}

pub enum PdColor {
    Solid(LCDSolidColor),
    Pattern(Arc<LCDPattern>),
}

#[derive(Component)]
pub struct BackgroundColor(pub PdColor);

#[derive(Component)]
pub struct BorderColor(pub PdColor);
#[derive(Component)]
pub struct BorderWidth(pub f32);

#[derive(Component)]
#[require(Sprite)]
pub struct ComputedNode {
    pub rect: PDRect,
}

fn render_node(
    mut query: Query<
        (
            &mut Sprite,
            &ComputedNode,
            Option<&BackgroundColor>,
            Option<(&BorderWidth, &BorderColor)>,
        ),
        (
            Or<(
                Changed<ComputedNode>,
                Changed<BackgroundColor>,
                Changed<BorderWidth>,
                Changed<BorderColor>,
            )>,
        ),
    >,
) {
    for (spr, node, color, border) in query.iter_mut() {
        let color = color.unwrap_or(&BackgroundColor(PdColor::Solid(LCDSolidColor::kColorClear)));
        let mut rect = PDRect {
            x: node.rect.x,
            y: node.rect.y,
            width: node.rect.width,
            height: node.rect.height,
        };
        
        // if let Some((border, ))
        // 
        // let bmp = Rc::make_mut(spr.bitmap.get_or_insert_with(|| {
        //     Bitmap::new(node.rect.width as c_int, node.rect.height as c_int, )
        // }));
    }
}
