#![no_std]
#![no_main]

mod framebuffer;

use core::panic::PanicInfo;

use bootloader_api::{BootInfo, entry_point};

use crate::framebuffer::init_writer;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();

    init_writer(framebuffer);

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
