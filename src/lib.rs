#![no_main]
#![no_std]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

pub fn busy_spin(iterations: usize) {
    for _ in 0..iterations {
        core::hint::spin_loop();
    }
}

pub struct Mutex<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

// SAFETY: Mutex<T> can be shared across threads when T: Send because the lock
// guarantees exclusive access to T — only one thread can access the inner data
// at a time.
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    #[must_use]
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    #[must_use]
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    #[must_use]
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self
            .lock
            // Success: Acquire to sync on all writes before previous unlock
            // Failure: No sync required
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(MutexGuard {
                lock: &self.lock,
                data: self.data.get(),
            })
        } else {
            None
        }
    }

    #[must_use]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        loop {
            if let Some(guard) = self.try_lock() {
                break guard;
            }
            // Relaxed as we do not need sync to check if lock to be free
            while self.lock.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }
    }
}

pub struct MutexGuard<'a, T> {
    lock: &'a AtomicBool,
    data: *mut T,
}

// SAFETY: MutexGuard can be sent to another thread when T: Send. The mutex
// guarantees exclusive access — only one guard exists at a time — so sending
// the guard transfers ownership of that exclusive access. The Drop impl uses
// Release ordering, ensuring writes are visible to the next acquirer regardless
// of which thread drops the guard. The MutexGuard has no dependencies on
// thread-local storage, the thread stack or similar
unsafe impl<T: Send> Send for MutexGuard<'_, T> {}

// SAFETY: MutexGuard is Sync when T is Sync. Sharing &MutexGuard across
// threads only allows &T access (via Deref). &mut T requires &mut MutexGuard,
// which Rust's borrowing rules prevent while &MutexGuard is shared.
unsafe impl<T: Sync> Sync for MutexGuard<'_, T> {}

impl<'a, T> core::ops::Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: when we have a MutexGuard the access is exclusive so giving
        // out a &data is safe
        unsafe { &*self.data }
    }
}

impl<'a, T> core::ops::DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: when we have a MutexGuard the access is exclusive so giving
        // out a &mut data is safe
        unsafe { &mut *self.data }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        // Release so all our writes will be visible on next lock
        self.lock.store(false, Ordering::Release);
    }
}

pub mod x86_64 {
    pub fn halt() -> ! {
        // SAFETY: cli/hlt are safe to execute in ring 0. As we run
        // single-threaded in ring0 no other process will 'sti'
        unsafe {
            core::arch::asm!("cli", "2: hlt", "jmp 2b", options(nomem, nostack, noreturn));
        }
    }

    pub fn reset() -> ! {
        // SAFETY: 0x64 is the keyboard controller command port.
        // Command 0xFE pulses the CPU reset line.
        unsafe {
            outb(0x64, 0xFE);
        }
        halt();
    }

    /// # Safety
    ///
    /// Caller must ensure the port access is valid and won't cause undefined
    /// behavior (e.g., triggering DMA to invalid memory, corrupting hardware
    /// state).
    pub unsafe fn outb(port: u16, value: u8) {
        // SAFETY: Caller needs to ensure guarantees are met
        unsafe {
            core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nostack, preserves_flags));
        }
    }

    /// # Safety
    ///
    /// Caller must ensure the port access is valid and won't cause undefined
    /// behavior (e.g., triggering DMA to invalid memory, corrupting hardware
    /// state).
    #[must_use]
    pub unsafe fn inb(port: u16) -> u8 {
        let value: u8;
        // SAFETY: Caller needs to ensure guarantees are met
        unsafe {
            core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nostack, preserves_flags));
        }
        value
    }
}

pub mod qemu {
    #[repr(u32)]
    pub enum QemuExitCode {
        Success = 0x10,
        Failure = 0x11,
    }

    pub fn qemu_exit(exit_code: QemuExitCode) -> ! {
        // SAFETY: 0xF4 is the port for QEMU exit.
        // 'hlt' is safe to execute in ring 0.
        unsafe {
            core::arch::asm!(
                "out dx, eax",
                "cli",
                "2: hlt",
                "jmp 2b",
                in("dx") 0xf4u16,
                in("eax") exit_code as u32,
                options(nostack, noreturn),
            );
        }
    }
}

pub mod serial;
pub mod vga;

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
    where
        T: Fn(),
    {
        fn run(&self) {
            serial_print!("{}...\t", core::any::type_name::<T>());
            self();
            serial_println!("[ok]");
        }
    }

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    qemu::qemu_exit(qemu::QemuExitCode::Success);
}

pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    use crate::qemu::{QemuExitCode, qemu_exit};
    use core::fmt::Write;

    // Brute-force access to serial to print panic message
    let mut port = crate::serial::SerialPort::new();
    writeln!(port, "[failed]").ok();
    writeln!(port, "Error: {}", info).ok();
    qemu_exit(QemuExitCode::Failure);
}

#[cfg(test)]
bootloader::entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    test_main();
    crate::qemu::qemu_exit(crate::qemu::QemuExitCode::Success);
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test_panic_handler(info)
}

#[cfg(test)]
mod tests {
}
