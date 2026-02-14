#![no_std]
#![no_main]

use bootloader::entry_point;
use core::panic::PanicInfo;
use kleinos::{
    qemu::{QemuExitCode, qemu_exit},
    serial_print, serial_println,
};

entry_point!(should_fail);

fn should_fail(_boot_info: &'static bootloader::BootInfo) -> ! {
    serial_print!("should_fail::should_fail...\t");
    assert_eq!(0, 1);
    serial_println!("[test did not panic]");
    qemu_exit(QemuExitCode::Failure);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    qemu_exit(QemuExitCode::Success);
}
