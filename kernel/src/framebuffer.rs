use core::fmt::{self, Write};

use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use noto_sans_mono_bitmap::{
    FontWeight, RasterHeight, get_raster, get_raster_width,
};
use spin::{Mutex, Once};
use x86_64::instructions::interrupts;

pub static WRITER: Once<Mutex<Writer>> = Once::new();

pub fn init_writer(framebuffer: &'static mut FrameBuffer) {
    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();

    let mut writer = Writer::new(buffer, info);
    writer.clear(Color::BLACK);
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
    ($color:expr) => {
        ($crate::framebuffer::_clear($color));
    };
}

#[macro_export]
macro_rules! set_color {
    ($fg:expr) => {
        $crate::framebuffer::_set_color($fg)
    };
    ($fg:expr, $bg:expr) => {
        $crate::framebuffer::_set_colors($fg, $bg)
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        WRITER.get().unwrap().lock().write_fmt(args).unwrap();
    })
}

#[doc(hidden)]
pub fn _clear(color: Color) {
    interrupts::without_interrupts(|| {
        WRITER.get().unwrap().lock().clear(color);
    })
}

#[doc(hidden)]
pub fn _set_color(fg: Color) {
    interrupts::without_interrupts(|| {
        WRITER.get().unwrap().lock().set_color(fg);
    })
}

#[doc(hidden)]
pub fn _set_colors(fg: Color, bg: Color) {
    interrupts::without_interrupts(|| {
        WRITER.get().unwrap().lock().set_colors(fg, bg);
    })
}

const CHAR_HEIGHT: usize = 16;
const LINE_HEIGHT: usize = 17;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0 };
    pub const CYAN: Color = Color { r: 0, g: 255, b: 255 };

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

pub struct Writer {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
    fg_color: Color,
    bg_color: Color,
}

impl Writer {
    fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
            fg_color: Color::WHITE,
            bg_color: Color::BLACK,
        }
    }

    pub fn set_color(&mut self, fg: Color) {
        self.fg_color = fg;
    }

    pub fn set_colors(&mut self, fg: Color, bg: Color) {
        self.fg_color = fg;
        self.bg_color = bg;
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, color: Color) {
        let pixel_offset = y * self.info.stride + x;
        let bpp = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bpp;
        let bytes = self.pixel_bytes(color);

        self.framebuffer[byte_offset..byte_offset + bpp]
            .copy_from_slice(&bytes[..bpp]);
    }

    pub fn scroll(&mut self) {
        let height = self.info.height;
        let stride = self.info.stride;
        let bpp = self.info.bytes_per_pixel;
        let row_bytes = stride * bpp;
        let scroll_bytes = LINE_HEIGHT * row_bytes;

        self.framebuffer.copy_within(scroll_bytes.., 0);

        let pixel = self.pixel_bytes(self.bg_color);
        let clear_start = (height - LINE_HEIGHT) * row_bytes;
        for chunk in self.framebuffer[clear_start..].chunks_exact_mut(bpp) {
            chunk.copy_from_slice(&pixel[..bpp]);
        }

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

    pub fn clear(&mut self, color: Color) {
        let bpp = self.info.bytes_per_pixel;
        let pixel = self.pixel_bytes(color);

        for chunk in self.framebuffer.chunks_exact_mut(bpp) {
            chunk.copy_from_slice(&pixel[..bpp]);
        }

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
                let intensity = *byte as u32;
                let blend = |fg: u8, bg: u8| -> u8 {
                    ((fg as u32 * intensity + bg as u32 * (255 - intensity))
                        / 255) as u8
                };
                let color = Color {
                    r: blend(self.fg_color.r, self.bg_color.r),
                    g: blend(self.fg_color.g, self.bg_color.g),
                    b: blend(self.fg_color.b, self.bg_color.b),
                };
                self.write_pixel(self.x_pos + col, self.y_pos + row, color);
            }
        }

        self.x_pos += width;
    }

    fn pixel_bytes(&self, color: Color) -> [u8; 4] {
        match self.info.pixel_format {
            PixelFormat::Rgb => [color.r, color.g, color.b, 0],
            PixelFormat::Bgr => [color.b, color.g, color.r, 0],
            PixelFormat::U8 => {
                let gray = ((color.r as u16 + color.g as u16 + color.b as u16)
                    / 3) as u8;
                [gray, 0, 0, 0]
            }
            _ => [color.r, color.g, color.b, 0],
        }
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
