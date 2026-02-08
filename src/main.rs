#![no_std]
#![no_main]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]

use core::fmt::Write;
use core::panic::PanicInfo;
use kleinos::qemu::{QemuExitCode, qemu_exit};
use kleinos::vga::{Color, ColorCode, ScreenChar};
use kleinos::x86_64;
use kleinos::{busy_spin, vga};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let red = ColorCode::new(Color::Red, Color::Black);

    if let Some(mut screen) = vga::SCREEN.try_lock() {
        screen.write(b'*', red, 0, 0);
    } else {
        let ch = ScreenChar {
            character: b'*',
            color: red,
        };
        // SAFETY: 0xb8000 is the VGA text buffer, a fixed physical address that
        // remains valid and mapped for the kernel's lifetime. We bypass the
        // lock because the panic may have occurred while holding it. The task
        // holding the lock will not be running again and the kernel will
        // terminate.
        unsafe {
            core::ptr::write_volatile(
                (0xb8000 as *mut ScreenChar).add((vga::BUFFER_HEIGHT - 1) * vga::BUFFER_WIDTH),
                ch,
            );
        }
    }

    x86_64::halt();
}

bootloader::entry_point!(kernel_main);

pub fn kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    let screen = &vga::SCREEN;
    writeln!(screen.lock(), "Hello, Rustaceans!").unwrap();

    busy_spin(100_000_000);
    qemu_exit(QemuExitCode::Success);
}
