use bevy_playdate::file::FileHandle;
use pd::fs::FileOptions;
use tiled::ResourceReader;

pub struct PlaydateReader;

impl ResourceReader for PlaydateReader {
    type Resource = FileHandle;

    type Error = no_std_io2::io::Error;

    fn read_from(&mut self, path: &tiled::ResourcePath) -> core::result::Result<Self::Resource, Self::Error> {
        FileHandle::open(path, FileOptions::kFileRead)
    }
}
