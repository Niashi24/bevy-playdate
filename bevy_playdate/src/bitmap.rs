use alloc::rc::Rc;
use playdate::api;
use playdate::graphics::bitmap::Bitmap;
use playdate::graphics::color::Color;
use playdate::sys::ffi::LCDBitmap;

// #[derive(Clone)]
// pub struct OwnedLCDBitmap {
//     ptr: *mut LCDBitmap,
// }
// 
// impl Drop for OwnedLCDBitmap {
//     fn drop(&mut self) {
//         
//     }
// }