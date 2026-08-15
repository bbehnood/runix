use core::fmt::{self, Write};

use spin::{Mutex, Once};
use uart_16550::{Config, Uart16550Tty, backend::PioBackend};

pub static SERIAL: Once<Mutex<Uart16550Tty<PioBackend>>> = Once::new();

pub fn init_writer() {
    let uart = unsafe {
        Uart16550Tty::new_port(0x3F8, Config::default())
            .expect("failed to initialize UART")
    };

    SERIAL.call_once(|| Mutex::new(uart));
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    SERIAL
        .get()
        .unwrap()
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
}
