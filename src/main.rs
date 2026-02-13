#![no_std]
#![no_main]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]

use kleinos::{hlt_loop, println};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\nPANIC: {}", info);
    hlt_loop();
}

bootloader::entry_point!(kernel_main);

pub fn kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    println!("Kernel starting...");

    kleinos::init();
    println!("Kernel init complete");

    hlt_loop();
}
