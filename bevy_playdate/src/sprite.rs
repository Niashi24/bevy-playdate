use alloc::rc::Rc;
use alloc::string::String;
use core::ptr::NonNull;
use bevy_app::{App, Plugin, PostUpdate};
use bevy_ecs::component::{Component, ComponentHooks, ComponentId, StorageType};
use bevy_ecs::entity::Entity;
use bevy_ecs::world::DeferredWorld;
use playdate::{api, println};
use playdate::graphics::api::Cache;
use playdate::graphics::bitmap::Bitmap;
use playdate::graphics::color::Color;
use playdate::graphics::{BitmapFlip, BitmapFlipExt, Graphics};
use playdate::sys::ffi::{LCDBitmap, LCDSolidColor, LCDSprite, PDRect};
use playdate::sys::traits::AsRaw;
use playdate::sprite::Sprite as PDSprite;
use derive_more::Deref;

pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        // todo: reflect component
        app.add_systems(
            PostUpdate,
            Sprite::draw_sprites,
        );
    }
}

#[derive(Component, Clone, Deref)]
#[component(on_add = add_to_display_list)]
#[component(on_replace = remove_from_display_list)]
pub struct Sprite {
    #[deref]
    spr: PDSprite,
    /// TODO: Replace with Handle
    pub bitmap: Option<Rc<Bitmap>>
}

fn add_to_display_list(w: DeferredWorld, e: Entity, _: ComponentId) {
    w.get::<Sprite>(e).unwrap().add_to_display_list()
}

fn remove_from_display_list(w: DeferredWorld, e: Entity, _: ComponentId) {
    w.get::<Sprite>(e).unwrap().remove_from_display_list()
}

// SAFETY: The Playdate is single-threaded.
// The component trait requires Send + Sync
unsafe impl Send for Sprite {}
unsafe impl Sync for Sprite {}

impl Sprite {
    /// Creates a new, empty Sprite
    pub fn new() -> Self {
        let spr = PDSprite::new();

        Sprite { spr, bitmap: None }
    }
    
    pub fn new_from_bitmap(bitmap: Rc<Bitmap>, flip: BitmapFlip) -> Self {
        let spr = PDSprite::new();
        spr.set_image(&*bitmap, flip);
        
        Self {
            spr,
            bitmap: Some(bitmap),
        }
    }
    
    pub fn new_from_draw(width: i32, height: i32, bg_color: Color, draw_fn: impl FnOnce(Graphics<Cache>)) -> Self {        
        let image = Bitmap::new(width, height, bg_color).unwrap();
        
        unsafe { api!(graphics).pushContext.unwrap()(image.as_raw()); }
        
        draw_fn(Graphics::Cached());

        unsafe { api!(graphics).popContext.unwrap()(); }
        
        Self::new_from_bitmap(Rc::new(image), BitmapFlip::Unflipped)
    }
    
    pub fn bitmap(&self) -> Option<Rc<Bitmap>> {
        self.bitmap.clone()
    }
    
    /// Add this sprite to the display list, so that it is drawn in the current scene.
    /// This is automatically called when inserting this into an entity.
    pub fn add_to_display_list(&self) {
        self.spr.add();
    }

    /// Remove this sprite to the display list, so that it is drawn in the current scene.
    /// This is automatically called when inserting this into an entity.
    pub fn remove_from_display_list(&self) {
        self.spr.remove();
    }

    /// System to draw all sprites to the screen. Calls [`playdate::sprite::draw_sprites`].
    /// 
    /// If your draw calls are not showing up, order that system after this one.
    #[inline]
    pub fn draw_sprites() {
        playdate::sprite::draw_sprites();
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::new()
    }
}
