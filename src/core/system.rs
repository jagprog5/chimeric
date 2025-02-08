use std::{collections::HashMap, ffi::CStr, num::NonZeroUsize, path::Path};

use sdl2::{
    image::Sdl2ImageContext,
    mixer::Sdl2MixerContext,
    rect::{FPoint, FRect, Point, Rect},
    render::{Canvas, Texture},
    ttf::Sdl2TtfContext,
    video::Window,
    AudioSubsystem, Sdl, VideoSubsystem,
};

use super::{
    font_system::font_system::FontSystem,
    render_system::{CanvasAndCreator, RenderSystem},
};

// use super::{audio_system::AudioSystem, render_system::{CanvasAndCreator, RenderSystem}};

/// core sdl2 system needed for the engine
pub struct System {
    pub image: Sdl2ImageContext,
    pub mixer: Sdl2MixerContext,
    pub ttf: Sdl2TtfContext,
    // dropped in member order stated
    pub video: VideoSubsystem,
    pub audio: AudioSubsystem,
    // dropped last
    pub sdl: Sdl,
}

impl System {
    pub fn new() -> Result<Self, String> {
        let sdl = sdl2::init()?;
        let video = sdl.video()?;
        let audio = sdl.audio()?;
        sdl2::mixer::open_audio(
            44_100,
            sdl2::mixer::AUDIO_S16LSB,
            sdl2::mixer::DEFAULT_CHANNELS,
            1_024,
        )?;
        sdl2::mixer::allocate_channels(8);

        Ok(System {
            sdl,
            video,
            audio,
            // empty flags - don't load any dynamic libs up front. they will be
            // loaded as needed the first time the respective file format is loaded
            image: sdl2::image::init(sdl2::image::InitFlag::empty())?,
            mixer: sdl2::mixer::init(sdl2::mixer::InitFlag::empty())?,
            ttf: sdl2::ttf::init().map_err(|e| e.to_string())?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChimericSystemSettings {
    pub num_point_sizes_per_font: NonZeroUsize,
    pub num_fonts: NonZeroUsize,
    pub num_textures_per_window: NonZeroUsize,
}

#[derive(Debug, Clone, Copy)]
pub struct CopyStruct {
    pub src: Option<Rect>,
    pub dst: Option<Rect>,
}

#[derive(Debug, Clone, Copy)]
pub struct CopyStructF {
    pub src: Option<Rect>,
    pub dst: Option<FRect>,
}

#[derive(Debug, Clone, Copy)]
pub struct CopyStructEx {
    pub src: Option<Rect>,
    pub dst: Option<Rect>,
    pub angle: f64,
    pub center: Point,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct CopyStructExF {
    pub src: Option<Rect>,
    pub dst: Option<FRect>,
    pub angle: f64,
    pub center: FPoint,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

/// abstraction above sdl systems (memory management, etc)
pub struct ChimericSystem<'sdl> {
    settings: ChimericSystemSettings,
    font_system: FontSystem<'sdl>,
    windows: HashMap<String, RenderSystem<'sdl>>,
    // pub sounds: AudioSystem<'sdl>,
    _system: &'sdl System,
}

impl<'sdl> ChimericSystem<'sdl> {
    pub fn new(system: &'sdl System, settings: ChimericSystemSettings) -> Self {
        Self {
            settings,
            font_system: FontSystem::new(
                &system.ttf,
                settings.num_point_sizes_per_font,
                settings.num_fonts,
            ),
            _system: system,
            windows: Default::default(),
            // sounds: AudioSystem::new(&system.audio),
        }
    }

    /// add a window to the app with a string key
    pub fn add_window(&mut self, window_name: &str, window: Window) -> Result<(), String> {
        let cc = CanvasAndCreator::new(window)?;
        let sys = RenderSystem::new(cc, self.settings.num_textures_per_window);
        let entry = self.windows.entry(window_name.into());
        match entry {
            std::collections::hash_map::Entry::Occupied(_occupied_entry) => Err(format!(
                "window \"{window_name}\" can't be created because it already exists"
            )),
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(sys);
                Ok(())
            }
        }
    }

    /// remove a window from the app by string key
    pub fn remove_window(&mut self, window_name: &str) -> Result<(), String> {
        match self.windows.remove(window_name) {
            Some(_v) => Ok(()),
            None => Err(format!(
                "window \"{window_name}\" can't be removed because it does not exist"
            )),
        }
    }

    pub fn present(&mut self) {
        self.windows.iter_mut().for_each(|v| v.1.present());
    }

    /// load the texture from the file path if its not in the cache; used to
    /// draw to the window specified by name. see Canvas::copy for more details
    pub fn copy<R1, R2>(
        &mut self,
        window_name: &str,
        path: &Path,
        src: R1,
        dst: R2,
    ) -> Result<(), String>
    where
        R1: Into<Option<Rect>>,
        R2: Into<Option<Rect>>,
    {
        let v = self.texture(window_name, path)?;
        v.1.copy(v.0, src, dst)
    }

    /// load the texture from the file path if its not in the cache; used to
    /// draw to the window specified by name. see Canvas::copy for more details
    pub fn copy_many<I>(
        &mut self,
        window_name: &str,
        path: &Path,
        copys: I,
    ) -> Result<(), String>
    where
        I: Iterator<Item = CopyStruct>,
    {
        let v = self.texture(window_name, path)?;
        for copy in copys.into_iter() {
            v.1.copy(v.0, copy.src, copy.dst)?;
        }
        Ok(())
    }

    /// load the texture from the file path if its not in the cache; used to
    /// draw to the window specified by name. see Canvas::copy for more details
    pub fn copy_f<R1, R2>(
        &mut self,
        window_name: &str,
        path: &Path,
        src: R1,
        dst: R2,
    ) -> Result<(), String>
    where
        R1: Into<Option<Rect>>,
        R2: Into<Option<FRect>>,
    {
        let v = self.texture(window_name, path)?;
        v.1.copy_f(v.0, src, dst)
    }

    /// load the texture from the file path if its not in the cache; used to
    /// draw to the window specified by name. see Canvas::copy for more details
    pub fn copy_many_f<I>(
        &mut self,
        window_name: &str,
        path: &Path,
        copys: I,
    ) -> Result<(), String>
    where
        I: Iterator<Item = CopyStructF>,
    {
        let v = self.texture(window_name, path)?;
        for copy in copys.into_iter() {
            v.1.copy_f(v.0, copy.src, copy.dst)?;
        }
        Ok(())
    }

    /// load the texture from the file path if its not in the cache; used to
    /// draw to the window specified by name. see Canvas::copy for more details
    pub fn copy_ex<R1, R2, P>(
        &mut self,
        window_name: &str,
        path: &Path,
        src: R1,
        dst: R2,
        angle: f64,
        center: P,
        flip_horizontal: bool,
        flip_vertical: bool,
    ) -> Result<(), String>
    where
        R1: Into<Option<Rect>>,
        R2: Into<Option<Rect>>,
        P: Into<Option<Point>>,
    {
        let v = self.texture(window_name, path)?;
        v.1.copy_ex(v.0, src, dst, angle, center, flip_horizontal, flip_vertical)
    }

    /// load the texture from the file path if its not in the cache; used to
    /// draw to the window specified by name. see Canvas::copy for more details
    pub fn copy_many_ex<I>(
        &mut self,
        window_name: &str,
        path: &Path,
        copys: I,
    ) -> Result<(), String>
    where
        I: Iterator<Item = CopyStructEx>,
    {
        let v = self.texture(window_name, path)?;
        for copy in copys.into_iter() {
            v.1.copy_ex(
                v.0,
                copy.src,
                copy.dst,
                copy.angle,
                copy.center,
                copy.flip_horizontal,
                copy.flip_vertical,
            )?;
        }
        Ok(())
    }

    /// load the texture from the file path if its not in the cache; used to
    /// draw to the window specified by name. see Canvas::copy for more details
    pub fn copy_ex_f<R1, R2, P>(
        &mut self,
        window_name: &str,
        path: &Path,
        src: R1,
        dst: R2,
        angle: f64,
        center: P,
        flip_horizontal: bool,
        flip_vertical: bool,
    ) -> Result<(), String>
    where
        R1: Into<Option<Rect>>,
        R2: Into<Option<FRect>>,
        P: Into<Option<FPoint>>,
    {
        let v = self.texture(window_name, path)?;
        v.1.copy_ex_f(v.0, src, dst, angle, center, flip_horizontal, flip_vertical)
    }

    /// load the texture from the file path if its not in the cache; used to
    /// draw to the window specified by name. see Canvas::copy for more details
    pub fn copy_many_ex_f<I>(
        &mut self,
        window_name: &str,
        path: &Path,
        copys: I,
    ) -> Result<(), String>
    where
        I: Iterator<Item = CopyStructExF>,
    {
        let v = self.texture(window_name, path)?;
        for copy in copys.into_iter() {
            v.1.copy_ex_f(
                v.0,
                copy.src,
                copy.dst,
                copy.angle,
                copy.center,
                copy.flip_horizontal,
                copy.flip_vertical,
            )?;
        }
        Ok(())
    }

    /// create the rendered text if needed, load the font as needed; used to
    /// draw to the window specified by name
    pub fn copy_text<R1, R2>(
        &mut self,
        window_name: &str,
        font_file: &Path,
        point_size: u16,
        text: &CStr,
        wrap_width: Option<u32>,
        src: R1,
        dst: R2,
    ) -> Result<(), String>
    where
        R1: Into<Option<Rect>>,
        R2: Into<Option<Rect>>,
    {
        let v = self.text(window_name, font_file, point_size, text, wrap_width)?;
        v.1.copy(v.0, src, dst)
    }

    /// create the rendered text if needed, load the font as needed; used to
    /// draw to the window specified by name
    pub fn copy_text_f<'me, R1, R2>(
        &'me mut self,
        window_name: &str,
        font_file: &Path,
        point_size: u16,
        text: &CStr,
        wrap_width: Option<u32>,
        src: R1,
        dst: R2,
    ) -> Result<(), String>
    where
        'me: 'sdl,
        R1: Into<Option<Rect>>,
        R2: Into<Option<FRect>>,
    {
        let v = self.text(window_name, font_file, point_size, text, wrap_width)?;
        v.1.copy_f(v.0, src, dst)
    }

    /// create the rendered text if needed, load the font as needed; used to
    /// draw to the window specified by name
    pub fn copy_text_ex<'me, R1, R2, P>(
        &'me mut self,
        window_name: &str,
        font_file: &Path,
        point_size: u16,
        text: &CStr,
        wrap_width: Option<u32>,
        src: R1,
        dst: R2,
        angle: f64,
        center: P,
        flip_horizontal: bool,
        flip_vertical: bool,
    ) -> Result<(), String>
    where
        'me: 'sdl,
        R1: Into<Option<Rect>>,
        R2: Into<Option<Rect>>,
        P: Into<Option<Point>>,
    {
        let v = self.text(window_name, font_file, point_size, text, wrap_width)?;
        v.1.copy_ex(v.0, src, dst, angle, center, flip_horizontal, flip_vertical)
    }

    /// create the rendered text if needed, load the font as needed; used to
    /// draw to the window specified by name
    pub fn copy_text_ex_f<'me, R1, R2, P>(
        &'me mut self,
        window_name: &str,
        font_file: &Path,
        point_size: u16,
        text: &CStr,
        wrap_width: Option<u32>,
        src: R1,
        dst: R2,
        angle: f64,
        center: P,
        flip_horizontal: bool,
        flip_vertical: bool,
    ) -> Result<(), String>
    where
        'me: 'sdl,
        R1: Into<Option<Rect>>,
        R2: Into<Option<FRect>>,
        P: Into<Option<FPoint>>,
    {
        let v = self.text(window_name, font_file, point_size, text, wrap_width)?;
        v.1.copy_ex_f(v.0, src, dst, angle, center, flip_horizontal, flip_vertical)
    }

    // =========================== base functions ==============================

    /// load the texture from the file path if its not in the cache; used to
    /// draw to the window specified by name
    fn texture(
        &mut self,
        window_name: &str,
        path: &Path,
    ) -> Result<(&mut Texture, &mut Canvas<Window>), String>
    {
        match self.windows.get_mut(window_name.into()) {
            None => Err(format!(
                "can't get texture; window \"{window_name}\" does not exist"
            )),
            Some(window) => window.texture(path),
        }
    }

    /// create the texture for the rendered font, load the font as needed; used
    /// to draw to the window specified by name
    fn text(
        &mut self,
        window_name: &str,
        font_file: &Path,
        point_size: u16,
        text: &CStr,
        wrap_width: Option<u32>,
    ) -> Result<(&mut Texture, &mut Canvas<Window>), String> {
        match self.windows.get_mut(window_name.into()) {
            None => Err(format!(
                "can't get texture; window \"{window_name}\" does not exist"
            )),
            Some(window) => window.text(
                &mut self.font_system,
                font_file,
                point_size,
                text,
                wrap_width,
            ),
        }
    }
}
