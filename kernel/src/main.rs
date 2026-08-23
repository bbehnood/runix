#![no_std]
#![no_main]

use core::panic::PanicInfo;

use bootloader_api::{BootInfo, entry_point};
use kernel::{
    QemuExitCode, clear_screen, framebuffer::Color, println, serial_println,
    set_color,
};
use x86_64::instructions;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(boot_info);

    clear_screen!(Color::RED);
    clear_screen!(Color::BLUE);

    set_color!(Color::WHITE, Color::BLUE);

    println!("Hello, World!");

    loop {
        instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("PANIC: {}", info);
    kernel::exit_qemu(QemuExitCode::Failed);
}
