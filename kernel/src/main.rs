#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod framebuffer;
mod interrupts;
mod serial;

use core::panic::PanicInfo;

use bootloader_api::{BootInfo, entry_point};
use x86_64::instructions::{nop, port::Port};

use crate::{
    framebuffer::init_writer, interrupts::init_idt, serial::init_serial,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    loop {
        nop();
    }
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();

    init_writer(framebuffer);
    init_serial();
    init_idt();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("PANIC: {}", info);
    exit_qemu(QemuExitCode::Failed);
}
