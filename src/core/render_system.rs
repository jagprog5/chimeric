use std::{ffi::CStr, marker::PhantomData, num::NonZeroUsize, path::Path};

use lru::LruCache;
use sdl2::{
    image::LoadTexture,
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
};

use super::{
    font_system::font_system::FontSystem, render_system_txt_key::FileOrRenderedTextKey,
};

/// textures must only be used with their originating canvas + creator. this
/// provides a tight coupling between those components
pub struct CanvasAndCreator {
    pub canvas: Canvas<Window>,
    pub creator: TextureCreator<WindowContext>,
}

impl CanvasAndCreator {
    pub fn new(window: Window) -> Result<Self, String> {
        let canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;
        let creator = canvas.texture_creator();
        Ok(Self { canvas, creator })
    }
}

/// manages loading and unloading of textures, and rendering text
pub struct RenderSystem<'sdl> {
    /// using unsafe_textures features, but that's ok; the creator and textures
    /// all live in the same struct - no realistic opportunity for misuse
    textures: LruCache<FileOrRenderedTextKey, Texture>,
    /// dropped after textures are dropped
    cc: CanvasAndCreator,
    _phantom: PhantomData<&'sdl ()>,
}

impl<'sdl> RenderSystem<'sdl> {
    pub fn new(cc: CanvasAndCreator, num_loaded_textures: NonZeroUsize) -> Self {
        Self {
            cc,
            textures: LruCache::new(num_loaded_textures),
            _phantom: Default::default(),
        }
    }

    pub fn present(&mut self) {
        self.cc.canvas.present();
    }

    /// create the texture for the rendered font, load the font as needed
    ///
    /// returns the loaded texture and the canvas to draw it on. note that
    /// changes to the texture (color mod, etc) may be retained to future calls
    pub fn text(
        &mut self,
        font_system: &mut FontSystem,
        font_file: &Path,
        point_size: u16,
        text: &CStr,
        wrap_width: Option<u32>,
    ) -> Result<(&mut Texture, &mut Canvas<Window>), String>
    {
        let key = match wrap_width {
            Some(wrap_width) => FileOrRenderedTextKey::from_rendered_wrapped_text(
                text, font_file, point_size, wrap_width,
            ),
            None => FileOrRenderedTextKey::from_rendered_text(text, font_file, point_size),
        };

        Ok((
            self.textures
                .try_get_or_insert_mut(key, || -> Result<Texture, String> {
                    let surface = font_system.render(font_file, point_size, text, wrap_width)?;
                    self.cc
                        .creator
                        .create_texture_from_surface(surface)
                        .map_err(|e| e.to_string())
                })?,
            &mut self.cc.canvas,
        ))
    }

    /// load the texture from the file path if its not in the cache
    ///
    /// returns the loaded texture and the canvas to draw it on. note that
    /// changes to the texture (color mod, etc) may be retained to future calls
    pub fn texture(&mut self, path: &Path) -> Result<(&mut Texture, &mut Canvas<Window>), String> {
        Ok((
            self.textures
                .try_get_or_insert_mut(FileOrRenderedTextKey::from_path(path), || {
                    self.cc.creator.load_texture(path)
                })?,
            &mut self.cc.canvas,
        ))
    }
}
