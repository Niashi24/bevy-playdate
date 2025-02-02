use core::ffi::c_void;
use core::ffi::c_int;
use alloc::ffi::CString;

use no_std_io2::io::{self, *};
use playdate::sys;
use playdate::{fs::{api::{Api, Default}, File}, sys::traits::AsRaw};
use derive_more::{Deref, DerefMut};
use playdate::sys::ffi::SDFile;
use playdate::sys::ffi::FileOptions;

pub struct FileHandle {
    handle: *mut SDFile,
}

impl FileHandle {
    pub fn open(path: &str, mode: FileOptions) -> io::Result<Self> {
        let c_path = CString::new(path).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"))?;
        let handle = unsafe { sys::api!(file).open.unwrap()(c_path.as_ptr(), mode) };
        if handle.is_null() {
            Err(io::Error::new(io::ErrorKind::NotFound, "Failed to open file"))
        } else {
            Ok(FileHandle { handle })
        }
    }
}

impl Read for FileHandle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let result = unsafe { sys::api!(file).read.unwrap()(self.handle, buf.as_mut_ptr() as *mut c_void, buf.len() as u32) };
        if result < 0 {
            Err(io::Error::new(io::ErrorKind::Other, "Read error"))
        } else {
            Ok(result as usize)
        }
    }
}

impl Write for FileHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let result = unsafe { sys::api!(file).write.unwrap()(self.handle, buf.as_ptr() as *const c_void, buf.len() as u32) };
        if result < 0 {
            Err(io::Error::new(io::ErrorKind::Other, "Write error"))
        } else {
            Ok(result as usize)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let result = unsafe { sys::api!(file).flush.unwrap()(self.handle) };
        if result < 0 {
            Err(io::Error::new(io::ErrorKind::Other, "Flush error"))
        } else {
            Ok(())
        }
    }
}

impl Seek for FileHandle {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let (offset, whence) = match pos {
            SeekFrom::Start(n) => (n as c_int, 0),
            SeekFrom::End(n) => (n as c_int, 2),
            SeekFrom::Current(n) => (n as c_int, 1),
        };
        let result = unsafe { sys::api!(file).seek.unwrap()(self.handle, offset, whence) };
        if result < 0 {
            Err(io::Error::new(io::ErrorKind::Other, "Seek error"))
        } else {
            Ok(result as u64)
        }
    }
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        unsafe { sys::api!(file).close.unwrap()(self.handle) };
    }
}
