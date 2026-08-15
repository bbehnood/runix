use core::fmt::{self, Write};

use bootloader_api::{info::FrameBuffer, info::FrameBufferInfo};
use noto_sans_mono_bitmap::{
    FontWeight, RasterHeight, get_raster, get_raster_width,
};
use spin::{Mutex, Once};

pub static WRITER: Once<Mutex<Writer>> = Once::new();

pub fn init_writer(framebuffer: &'static mut FrameBuffer) {
    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();

    let mut writer = Writer::new(buffer, info);
    writer.clear();
    WRITER.call_once(|| Mutex::new(writer));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::framebuffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! clear_screen {
    () => {
        ($crate::framebuffer::_clear())
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.get().unwrap().lock().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _clear() {
    WRITER.get().unwrap().lock().clear();
}

const CHAR_HEIGHT: usize = 16;
const LINE_HEIGHT: usize = 17;

pub struct Writer {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl Writer {
    fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        Self { framebuffer, info, x_pos: 0, y_pos: 0 }
    }
}

impl Writer {
    pub fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * self.info.stride + x;
        let color = [intensity, intensity, intensity, 0];
        let bpp = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bpp;
        self.framebuffer[byte_offset..byte_offset + bpp]
            .copy_from_slice(&color[..bpp]);
    }

    pub fn scroll(&mut self) {
        let height = self.info.height;
        let stride = self.info.stride;
        let bpp = self.info.bytes_per_pixel;

        let row_bytes = stride * bpp;
        let scroll_bytes = LINE_HEIGHT * row_bytes;

        self.framebuffer.copy_within(scroll_bytes.., 0);

        let clear_start = (height - LINE_HEIGHT) * row_bytes;

        self.framebuffer[clear_start..].fill(0);

        self.y_pos = height - LINE_HEIGHT;
    }

    pub fn newline(&mut self) {
        self.x_pos = 0;

        if self.y_pos + LINE_HEIGHT + CHAR_HEIGHT > self.info.height {
            self.scroll();
        } else {
            self.y_pos += LINE_HEIGHT;
        }
    }

    pub fn clear(&mut self) {
        self.framebuffer.fill(0);
        self.x_pos = 0;
        self.y_pos = 0;
    }

    pub fn write_char(&mut self, c: char) {
        if c == '\n' {
            self.newline();
            return;
        }

        let width = get_raster_width(FontWeight::Regular, RasterHeight::Size16);
        if self.x_pos + width >= self.info.width {
            self.newline();
        }

        let raster = get_raster(c, FontWeight::Regular, RasterHeight::Size16)
            .unwrap_or_else(|| {
                get_raster(' ', FontWeight::Regular, RasterHeight::Size16)
                    .unwrap()
            });

        for (row, bytes) in raster.raster().iter().enumerate() {
            for (col, byte) in bytes.iter().enumerate() {
                self.write_pixel(self.x_pos + col, self.y_pos + row, *byte);
            }
        }

        self.x_pos += width;
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }

        Ok(())
    }
}
