use alloc::rc::Rc;
use alloc::string::String;
use core::ptr::NonNull;
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

#[derive(Component)]
#[component(on_add = add_to_display_list)]
#[component(on_replace = remove_from_display_list)]
pub struct Sprite {
    ptr: NonNull<LCDSprite>,
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
    pub fn new() -> Self {
        let ptr = NonNull::new(unsafe { api!(sprite).newSprite.unwrap()() }).unwrap();

        Sprite { ptr, bitmap: None }
    }
    
    pub fn new_from_bitmap(bitmap: Rc<Bitmap>, flip: BitmapFlip) -> Self {
        let ptr = NonNull::new(unsafe { api!(sprite).newSprite.unwrap()() }).unwrap();
        
        unsafe {
            api!(sprite).setImage.unwrap()(ptr.as_ptr(), bitmap.as_raw(), flip);
        }
        
        // unsafe {
        //     let (w, h) = bitmap.size();
        //     for y in 0..h {
        //         println!("{}", (0..w).map(|x| match bitmap.pixel_at(x, y) {
        //             LCDSolidColor::kColorBlack => "X",
        //             LCDSolidColor::kColorWhite => "O",
        //             LCDSolidColor::kColorClear => " ",
        //             LCDSolidColor::kColorXOR => "?",
        //         }).collect::<String>());
        //     }
        // }
        
        Self {
            ptr,
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
    
    pub unsafe fn from_ptr(ptr: NonNull<LCDSprite>, bitmap: Option<Rc<Bitmap>>) -> Self {
        Sprite { ptr, bitmap }
    }
    
    pub unsafe fn as_raw(&self) -> NonNull<LCDSprite> {
        self.ptr
    }
    
    pub fn bitmap(&self) -> Option<Rc<Bitmap>> {
        self.bitmap.clone()
    }
    
    /// Add this sprite to the display list, so that it is drawn in the current scene.
    /// This is automatically called when inserting this into an entity.
    pub fn add_to_display_list(&self) {
        unsafe {
            api!(sprite).addSprite.unwrap()(self.ptr.as_ptr());
        }
    }

    /// Remove this sprite to the display list, so that it is drawn in the current scene.
    /// This is automatically called when inserting this into an entity.
    pub fn remove_from_display_list(&self) {
        println!("removed");
        unsafe {
            api!(sprite).removeSprite.unwrap()(self.ptr.as_ptr());
        }
    }
    
    pub fn move_to(&mut self, x: f32, y: f32) {
        unsafe {
            api!(sprite).moveTo.unwrap()(self.ptr.as_ptr(), x, y);
        }
    }
    
    pub fn set_center(&mut self, x: f32, y: f32) {
        unsafe {
            api!(sprite).setCenter.unwrap()(self.ptr.as_ptr(), x, y);
        }
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Sprite {
    fn drop(&mut self) {
        unsafe { api!(sprite).freeSprite.unwrap()(self.ptr.as_ptr()) };
    }
}

impl Clone for Sprite {
    fn clone(&self) -> Self {
        unsafe {
            Self::from_ptr(NonNull::new(api!(sprite).copy.unwrap()(self.ptr.as_ptr())).unwrap(), self.bitmap.clone())
        }
    }
}