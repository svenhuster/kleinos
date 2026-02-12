#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn qemu_exit(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0xf4);

    // SAFETY: 0xf4 used above is the port configured for QEMU
    // exit. If it was not configured, it would be ignored and we end
    // up busy-spinning instead.
    unsafe {
        port.write(exit_code as u32);
    };

    loop {
        core::hint::spin_loop();
    }
}
