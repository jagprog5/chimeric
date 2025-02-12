use std::{ffi::{c_void, CStr}, marker::PhantomData, rc::Rc};

use sdl2::{get_error, libc::{c_int, c_uint}, surface::Surface, sys::{ttf, SDL_Color, SDL_RWops, SDL_Surface}, ttf::{FontStyle, GlyphMetrics, Hinting, Sdl2TtfContext}};

// rust-sdl2 wasn't sufficient. needed to model a Rc holding the font data
pub struct Font<'ttf> {
    raw: *mut ttf::TTF_Font,
    rwops: *mut SDL_RWops,
    marker: PhantomData<&'ttf ()>,
    font_file_content: Rc<Box<[u8]>>,
}

impl<'ttf> Font<'ttf> {
    pub fn new(
        _ttf: &'ttf Sdl2TtfContext,
        point_size: u16,
        font_file_content: Rc<Box<[u8]>>,
    ) -> Result<Self, String> {
        let clone = font_file_content.clone();

        let rwops = unsafe { sdl2::sys::SDL_RWFromConstMem(font_file_content.as_ptr() as *const c_void, font_file_content.len() as c_int) };
        if (rwops as *mut ()).is_null() {
            return Err(get_error())
        }

        let raw = unsafe { ttf::TTF_OpenFontRW(rwops, 0, point_size as c_int) };
        if (raw as *mut ()).is_null() {
            return Err(get_error())
        }
        Ok(Self {
            rwops,
            raw,
            marker: PhantomData,
            font_file_content: clone,
        })
    }

    pub fn get_content(&self) -> &Rc<Box<[u8]>> {
        &self.font_file_content
    }

    /// Returns the underlying C font object.
    // this can prevent introducing UB until
    // https://github.com/rust-lang/rust-clippy/issues/5953 is fixed
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub unsafe fn raw(&self) -> *mut ttf::TTF_Font {
        self.raw
    }

    pub fn render(&self, text: &CStr, wrap_width: Option<u32>) -> Result<Surface, String> {
        unsafe {
            let white = SDL_Color {
                r: 0xFF,
                g: 0xFF,
                b: 0xFF,
                a: 0xFF,
            };
            let surface: *mut SDL_Surface = match wrap_width {
                Some(wrap_width) => sdl2::sys::ttf::TTF_RenderUTF8_Blended_Wrapped(
                    self.raw(),
                    text.as_ptr(),
                    white,
                    wrap_width,
                ),
                None => {
                    sdl2::sys::ttf::TTF_RenderUTF8_Blended(self.raw(), text.as_ptr(), white)
                },
            };
            if (surface as *mut ()).is_null() {
                return Err(get_error())
            }
            Ok(Surface::from_ll(surface))
        }
    }

    /// Returns the width and height of the given text when rendered using this
    /// font.
    pub fn size_of(&self, text: &CStr) -> Result<(u32, u32), String> {
        let (res, size) = unsafe {
            let mut w = 0; // mutated by C code
            let mut h = 0; // mutated by C code
            let ret = ttf::TTF_SizeUTF8(self.raw, text.as_ptr(), &mut w, &mut h);
            (ret, (w as u32, h as u32))
        };
        if res == 0 {
            Ok(size)
        } else {
            Err(get_error())
        }
    }

    /// Returns the font's style flags.
    pub fn get_style(&self) -> FontStyle {        
        unsafe {
            let raw = ttf::TTF_GetFontStyle(self.raw);
            FontStyle::from_bits_truncate(raw as i32)
        }
    }

    /// Sets the font's style flags.
    pub fn set_style(&mut self, styles: FontStyle) {
        unsafe { ttf::TTF_SetFontStyle(self.raw, styles.bits() as c_int) }
    }

    /// Returns the width of the font's outline.
    pub fn get_outline_width(&self) -> u16 {
        unsafe { ttf::TTF_GetFontOutline(self.raw) as u16 }
    }

    /// Sets the width of the font's outline.
    pub fn set_outline_width(&mut self, width: u16) {
        unsafe { ttf::TTF_SetFontOutline(self.raw, width as c_int) }
    }

    /// Returns the font's freetype hints.
    pub fn get_hinting(&self) -> Hinting {
        unsafe {
            match ttf::TTF_GetFontHinting(self.raw) as c_uint {
                ttf::TTF_HINTING_NORMAL => Hinting::Normal,
                ttf::TTF_HINTING_LIGHT => Hinting::Light,
                ttf::TTF_HINTING_MONO => Hinting::Mono,
                ttf::TTF_HINTING_NONE | _ => Hinting::None,
            }
        }
    }

    /// Sets the font's freetype hints.
    pub fn set_hinting(&mut self, hinting: Hinting) {
        unsafe { ttf::TTF_SetFontHinting(self.raw, hinting as c_int) }
    }

    /// Returns whether the font is kerning.
    pub fn get_kerning(&self) -> bool {
        unsafe { ttf::TTF_GetFontKerning(self.raw) != 0 }
    }

    /// Sets whether the font should use kerning.
    pub fn set_kerning(&mut self, kerning: bool) {
        unsafe { ttf::TTF_SetFontKerning(self.raw, kerning as c_int) }
    }

    pub fn height(&self) -> i32 {
        //! Get font maximum total height.
        unsafe { ttf::TTF_FontHeight(self.raw) as i32 }
    }

    /// Returns the font's highest ascent (height above base).
    pub fn ascent(&self) -> i32 {
        unsafe { ttf::TTF_FontAscent(self.raw) as i32 }
    }

    /// Returns the font's lowest descent (height below base).
    /// This is a negative number.
    pub fn descent(&self) -> i32 {
        unsafe { ttf::TTF_FontDescent(self.raw) as i32 }
    }

    /// Returns the recommended line spacing for text rendered with this font.
    pub fn recommended_line_spacing(&self) -> i32 {
        unsafe { ttf::TTF_FontLineSkip(self.raw) as i32 }
    }

    /// Returns the number of faces in this font.
    pub fn face_count(&self) -> u16 {
        unsafe { ttf::TTF_FontFaces(self.raw) as u16 }
    }

    /// Returns whether the font is monospaced.
    pub fn face_is_fixed_width(&self) -> bool {
        unsafe { ttf::TTF_FontFaceIsFixedWidth(self.raw) != 0 }
    }

    /// Returns the family name of the current font face without doing any heap allocations.
    pub fn face_family_name_borrowed(&self) -> Option<&'ttf CStr> {
        unsafe {
            // not owns buffer
            let cname = ttf::TTF_FontFaceFamilyName(self.raw);
            if cname.is_null() {
                None
            } else {
                Some(CStr::from_ptr(cname))
            }
        }
    }

    /// Returns the family name of the current font face.
    pub fn face_family_name(&self) -> Option<String> {
        self.face_family_name_borrowed()
            .map(|cstr| String::from_utf8_lossy(cstr.to_bytes()).into_owned())
    }

    /// Returns the name of the current font face without doing any heap allocations.
    pub fn face_style_name_borrowed(&self) -> Option<&'ttf CStr> {
        unsafe {
            let cname = ttf::TTF_FontFaceStyleName(self.raw);
            if cname.is_null() {
                None
            } else {
                Some(CStr::from_ptr(cname))
            }
        }
    }

    /// Returns the name of the current font face.
    pub fn face_style_name(&self) -> Option<String> {
        self.face_style_name_borrowed()
            .map(|cstr| String::from_utf8_lossy(cstr.to_bytes()).into_owned())
    }

    /// Returns the index of the given character in this font face.
    pub fn find_glyph(&self, ch: char) -> Option<u16> {
        unsafe {
            let ret = ttf::TTF_GlyphIsProvided(self.raw, ch as u16);
            if ret == 0 {
                None
            } else {
                Some(ret as u16)
            }
        }
    }

    /// Returns the glyph metrics of the given character in this font face.
    pub fn find_glyph_metrics(&self, ch: char) -> Option<GlyphMetrics> {
        let mut minx = 0;
        let mut maxx = 0;
        let mut miny = 0;
        let mut maxy = 0;
        let mut advance = 0;

        let ret = unsafe {
            ttf::TTF_GlyphMetrics(
                self.raw,
                ch as u16,
                &mut minx,
                &mut maxx,
                &mut miny,
                &mut maxy,
                &mut advance,
            )
        };
        if ret == 0 {
            Some(GlyphMetrics {
                minx,
                maxx,
                miny,
                maxy,
                advance,
            })
        } else {
            None
        }
    }
}

impl<'ttf> Drop for Font<'ttf> {
    fn drop(&mut self) {
        let ret = unsafe { ((*self.rwops).close.unwrap())(self.rwops) };
        if ret != 0 {
            panic!("{}", get_error());
        }

        unsafe {
            // avoid close font after quit()
            if ttf::TTF_WasInit() == 1 {
                ttf::TTF_CloseFont(self.raw);
            }
        }
    }
}