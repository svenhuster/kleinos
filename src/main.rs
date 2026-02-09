#![no_std]
#![no_main]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use kleinos::qemu::{QemuExitCode, qemu_exit};
use kleinos::vga::{Color, ColorCode, ScreenChar};
use kleinos::x86_64;
use kleinos::{busy_spin, println};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Check if the panic handler can acquire the lock to see if we paniced
    // trying to write to the screen. Alternatively, just write a red '*' in the
    // top left corner.
    if let Some(screen) = kleinos::vga::SCREEN.try_lock() {
        drop(screen);
        println!("{}", info);
    } else {
        // SAFETY: 0xb8000 is the VGA text buffer, a fixed physical address that
        // remains valid and mapped for the kernel's lifetime. We bypass the
        // lock because the panic may have occurred while holding it. The task
        // holding the lock will not be running again and the kernel will
        // terminate.
        unsafe {
            let ch = ScreenChar {
                character: b'*',
                color: ColorCode::new(Color::Red, Color::Black),
            };
            core::ptr::write_volatile(0xb8000 as *mut ScreenChar, ch);
        }
    }

    x86_64::halt();
}

bootloader::entry_point!(kernel_main);

pub fn kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    #[cfg(test)]
    test_main();

    println!("Hello, Rustaceans!");

    busy_spin(100_000_000);
    qemu_exit(QemuExitCode::Success);
}

#[cfg(test)]
mod tests {
    use kleinos::{
        print, println,
        qemu::{QemuExitCode, qemu_exit},
    };

    pub fn test_runner(tests: &[&dyn Fn()]) {
        println!("Running {} tests", tests.len());
        for test in tests {
            test();
        }
        qemu_exit(QemuExitCode::Success);
    }

    #[test_case]
    fn trivial_assertion() {
        print!("trivial assertion... ");
        assert_eq!(1, 1);
        println!("[ok]");
    }
}
