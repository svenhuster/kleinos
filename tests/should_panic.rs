#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(panic_test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::entry_point;
use core::panic::PanicInfo;
use kleinos::{
    qemu::{QemuExitCode, qemu_exit},
    serial, serial_print, serial_println,
};

entry_point!(test_kernel_main);

fn test_kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    serial::SERIAL1.lock().init();
    test_main();
    loop {
        x86_64::instructions::hlt();
    }
}

pub trait PanicTestable {
    fn run(&self);
}

impl<T> PanicTestable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[test did not panic]");
    }
}

pub fn panic_test_runner(tests: &[&dyn PanicTestable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
        qemu_exit(QemuExitCode::Failure);
    }
    qemu_exit(QemuExitCode::Success);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    qemu_exit(QemuExitCode::Success);
}

#[test_case]
fn test_panic() {
    assert_eq!(0, 1);
}
