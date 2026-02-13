#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kleinos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::entry_point;
use core::panic::PanicInfo;
use kleinos::serial;

entry_point!(test_kernel_main);

fn test_kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    serial::SERIAL1.lock().init();
    test_main();

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kleinos::test_panic_handler(info);
}

#[test_case]
fn test_println() {
    use kleinos::println;
    println!("test_println output");
}
