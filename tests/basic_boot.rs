#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kleinos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::entry_point;
use core::panic::PanicInfo;
use kleinos::println;
use kleinos::{
    qemu::{QemuExitCode, qemu_exit},
    serial,
};

entry_point!(test_kernel_main);

fn test_kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    serial::PORT.lock().init();
    test_main();
    qemu_exit(QemuExitCode::Success);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kleinos::test_panic_handler(info);
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
