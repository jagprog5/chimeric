use std::{
    ffi::CStr,
    fs::File,
    io::Read,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    rc::Rc,
};

use lru::LruCache;
use sdl2::{
    surface::Surface,
    ttf::Sdl2TtfContext,
};

use super::font::Font;

pub struct FontSystem<'sdl> {
    // stored for creating a new value in font_objects
    num_font_objects_per_font: NonZeroUsize,
    num_font_objects: LruCache<PathBuf, LruCache<u16, Font<'sdl>>>,
    pub ttf: &'sdl Sdl2TtfContext,
}

impl<'sdl> FontSystem<'sdl> {
    pub fn new(
        ttf: &'sdl Sdl2TtfContext,
        num_font_objects_per_font: NonZeroUsize,
        min_loaded_fonts: NonZeroUsize,
    ) -> Self {
        Self {
            num_font_objects_per_font,
            num_font_objects: LruCache::new(min_loaded_fonts),
            ttf,
        }
    }

    /// render the text, loading the font file and or creating the font object
    /// if needed and not cached
    pub fn render(
        &mut self,
        font_file: &Path,
        point_size: u16,
        text: &CStr,
        wrap_width: Option<u32>,
    ) -> Result<Surface, String> {
        let font_objects_for_font = self
            .num_font_objects
            .get_or_insert_mut_ref(font_file, || LruCache::new(self.num_font_objects_per_font));

        let font_data_rc = match font_objects_for_font.peek_mru() {
            Some(font_object) => {
                // reuse the rc from one of the other objects
                font_object.1.get_content().clone()
            },
            None => {
                // this occurs because this font did not exist in the cache, and
                // a new entry was added to self.font_objects (but it doesn't
                // have any font objects in it yet)
                //
                // need to load the data in
                let mut font_file_contents: Vec<u8> = Vec::new();
                let mut file = File::open(font_file).map_err(|err| err.to_string())?;
                file.read_to_end(&mut font_file_contents)
                    .map_err(|err| err.to_string())?;
                Rc::new(font_file_contents.into_boxed_slice())
            }
        };

        let font_object = font_objects_for_font.try_get_or_insert(point_size, || {
            Font::new(&self.ttf, point_size, font_data_rc)
        })?;

        font_object.render(text, wrap_width)
    }
}
