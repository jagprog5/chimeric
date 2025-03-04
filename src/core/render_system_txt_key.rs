use std::{ffi::CStr, hash::Hasher, os::unix::ffi::OsStrExt, path::Path};

/// contains some encoding of the resource. used as lru key.
/// 
/// can contain one of three variants, identified by the first byte.
///
/// for texture from file
/// 
/// 0x00 + "/path/to/font"
/// 
/// for rendered text:
/// 
/// 0x01 + u16(16pt) + "some text\0" + "/path/to/font"
/// 
/// for rendered wrapping text:
///
/// 0x02 + u16(16pt) + u32(123pix) + "some text\0" + "/path/to/font"
pub struct FileOrRenderedTextKey {
    data: Vec<u8>,
}

impl PartialEq for FileOrRenderedTextKey {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for FileOrRenderedTextKey {}

impl std::hash::Hash for FileOrRenderedTextKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl FileOrRenderedTextKey {
    pub fn from_path(texture_path: &Path) -> Self {
        let mut data: Vec<u8> = Default::default();
        let data_len = 1 + texture_path.as_os_str().as_bytes().len();
        data.reserve_exact(data_len);
        unsafe { data.set_len(data_len); }
        data[0] = b'\x00';
        let mut index = 1;
        texture_path.as_os_str().as_bytes().iter().for_each(|&byte| {
            data[index] = byte;
            index += 1;
        });
        debug_assert_eq!(data.len(), data_len);
        Self {
            data
        }
    }

    pub fn from_rendered_text(text: &CStr, font_file: &Path, point_size: u16) -> Self {
        let text_bytes = text.to_bytes_with_nul();
        let point_size_bytes = point_size.to_le_bytes();
        let font_file_bytes = font_file.as_os_str().as_bytes();
        let data_len = 1 + size_of::<u16>() + text_bytes.len() + font_file_bytes.len();
        let mut data: Vec<u8> = Default::default();
        data.reserve_exact(data_len);
        unsafe { data.set_len(data_len); }
        let mut index = 0;
        data[index] = b'\x01';
        index += 1;
        data[index] = point_size_bytes[0];
        index += 1;
        data[index] = point_size_bytes[1];
        index += 1;
        text_bytes.iter().for_each(|&byte| {
            data[index] = byte;
            index += 1;
        });
        font_file_bytes.iter().for_each(|&byte| {
            data[index] = byte;
            index += 1;
        });
        debug_assert_eq!(data.len(), data_len);
        Self {
            data
        }
    }

    pub fn from_rendered_wrapped_text(text: &CStr, font_file: &Path, point_size: u16, wrap_width: u32) -> Self {
        let text_bytes = text.to_bytes_with_nul();
        let point_size_bytes = point_size.to_le_bytes();
        let wrap_width_bytes = wrap_width.to_le_bytes();
        let font_file_bytes = font_file.as_os_str().as_bytes();
        let data_len = 1 + size_of::<u16>() + size_of::<u32>() + text_bytes.len() + font_file_bytes.len();
        let mut data: Vec<u8> = Default::default();
        data.reserve_exact(data_len);
        unsafe { data.set_len(data_len); }
        let mut index = 0;
        data[index] = b'\x02';
        index += 1;
        data[index] = point_size_bytes[0];
        index += 1;
        data[index] = point_size_bytes[1];
        index += 1;
        data[index] = wrap_width_bytes[0];
        index += 1;
        data[index] = wrap_width_bytes[1];
        index += 1;
        data[index] = wrap_width_bytes[2];
        index += 1;
        data[index] = wrap_width_bytes[3];
        index += 1;
        text_bytes.iter().for_each(|&byte| {
            data[index] = byte;
            index += 1;
        });
        font_file_bytes.iter().for_each(|&byte| {
            data[index] = byte;
            index += 1;
        });
        debug_assert_eq!(data.len(), data_len);
        Self {
            data
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{path::{PathBuf, MAIN_SEPARATOR}, u32};

    use super::*;

    #[test]
    fn test_path() {
        let mut path = PathBuf::default();
        path.push("tester");
        path.push("abc");
        let s = FileOrRenderedTextKey::from_path(&path);

        let mut rhs: Vec<u8> = Default::default();
        rhs.push(b'\x00');
        rhs.extend_from_slice(b"tester");
        rhs.extend_from_slice(&[MAIN_SEPARATOR as u8]);
        rhs.extend_from_slice(b"abc");
        assert_eq!(s.data, rhs);
    }

    #[test]
    fn test_text() {
        let mut path = PathBuf::default();
        path.push("tester");
        path.push("abc");
        let s = FileOrRenderedTextKey::from_rendered_text(c"text", &path, 16);
        let mut rhs: Vec<u8> = Default::default();
        rhs.push(b'\x01');
        rhs.extend_from_slice(b"\x10\x00");
        rhs.extend_from_slice(b"text\0");
        rhs.extend_from_slice(b"tester");
        rhs.extend_from_slice(&[MAIN_SEPARATOR as u8]);
        rhs.extend_from_slice(b"abc");
        assert_eq!(s.data, rhs);
    }

    #[test]
    fn test_text_wrapped() {
        let mut path = PathBuf::default();
        path.push("tester");
        path.push("abc");
        let s = FileOrRenderedTextKey::from_rendered_wrapped_text(c"text", &path, 16, u32::MAX - 1);
        let mut rhs: Vec<u8> = Default::default();
        rhs.push(b'\x02');
        rhs.extend_from_slice(b"\x10\x00");
        rhs.extend_from_slice(b"\xFE\xFF\xFF\xFF");
        rhs.extend_from_slice(b"text\0");
        rhs.extend_from_slice(b"tester");
        rhs.extend_from_slice(&[MAIN_SEPARATOR as u8]);
        rhs.extend_from_slice(b"abc");
        assert_eq!(s.data, rhs);
    }
}
