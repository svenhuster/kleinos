#![no_std]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]

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
            core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
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
            core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
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
                options(nomem, nostack, noreturn),
            );
        }
    }
}

pub mod vga {
    use core::ptr::{read_volatile, write_volatile};

    #[derive(Clone, Copy)]
    #[repr(u8)]
    pub enum Color {
        Black = 0,
        Blue = 1,
        Green = 2,
        Cyan = 3,
        Red = 4,
        Magenta = 5,
        Brown = 6,
        LightGray = 7,
        DarkGray = 8,
        LightBlue = 9,
        LightGreen = 10,
        LightCyan = 11,
        LightRed = 12,
        Pink = 13,
        Yellow = 14,
        White = 15,
    }

    #[derive(Clone, Copy)]
    #[repr(transparent)]
    pub struct ColorCode(u8);

    impl ColorCode {
        #[must_use]
        pub const fn new(foreground: Color, background: Color) -> Self {
            Self((background as u8) << 4 | foreground as u8)
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ScreenChar {
        pub character: u8,
        pub color: ColorCode,
    }

    // Ensure ScreenChar layout matches the VGA buffer
    const _: () = assert!(core::mem::align_of::<ScreenChar>() == 1);
    const _: () = assert!(core::mem::size_of::<ScreenChar>() == 2);
    const _: () = assert!(core::mem::offset_of!(ScreenChar, character) == 0);
    const _: () = assert!(core::mem::offset_of!(ScreenChar, color) == 1);

    pub const BUFFER_HEIGHT: usize = 25;
    pub const BUFFER_WIDTH: usize = 80;

    pub static SCREEN: crate::Mutex<VgaScreen> = crate::Mutex::new(VgaScreen::new());

    pub struct VgaScreen {
        column: usize,
        color_code: ColorCode,
        buffer: *mut [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
    }

    // SAFETY: VgaScreen contains a raw pointer to the VGA buffer at 0xb8000,
    // which is memory-mapped hardware at a fixed physical address. This
    // memory is accessible from any CPU context and remains valid for the
    // kernel's lifetime.
    unsafe impl Send for VgaScreen {}

    impl VgaScreen {
        #[must_use]
        pub const fn new() -> Self {
            Self {
                column: 0,
                color_code: ColorCode::new(Color::LightGray, Color::Black),
                buffer: 0xb8000 as *mut _,
            }
        }
    }

    impl Default for VgaScreen {
        fn default() -> Self {
            Self::new()
        }
    }

    impl core::fmt::Write for VgaScreen {
        // Only ASCII will be printed properly on the VGA screen
        fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
            for ch in s.chars() {
                if ch.is_ascii() {
                    self.write_byte(ch as u8);
                } else {
                    self.write_byte(0xFE); // write the block char
                }
            }
            Ok(())
        }
    }

    impl VgaScreen {
        pub fn clear_line(&mut self) {
            for col in self.column..BUFFER_WIDTH {
                self.write(b' ', self.color_code, 0, col);
            }
        }

        pub fn new_line(&mut self) {
            // Move every line up one, top line is lost
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    // SAFETY: After initialization VgaScreen buffer points to
                    // the correct memory address for the VGA buffer. The loops
                    // ensure we are within the bounds of is memory region.
                    unsafe {
                        write_volatile(
                            &mut (*self.buffer)[row - 1][col],
                            read_volatile(&(*self.buffer)[row][col]),
                        );
                    }
                }
            }
            self.column = 0;
            self.clear_line();
        }

        pub fn write_byte(&mut self, byte: u8) {
            if self.column >= BUFFER_WIDTH {
                self.new_line();
            }
            if byte == b'\n' {
                self.new_line();
            } else {
                self.write(byte, self.color_code, 0, self.column);
                self.column += 1;
            }
        }

        pub fn write(&mut self, byte: u8, color: ColorCode, row: usize, col: usize) {
            if row >= BUFFER_HEIGHT || col >= BUFFER_WIDTH {
                panic!("access to vga buffer out of bounds");
            }

            // Writing starts from the bottom left of the screen
            let row = BUFFER_HEIGHT - row - 1;

            let ch = ScreenChar {
                character: byte,
                color,
            };

            // SAFETY: After initialization VgaScreen points to the VGA buffer
            // address. To get here the bounds check at the beginning of the fn
            // ensured that we are within the correct memory region.
            unsafe { write_volatile(&mut (*self.buffer)[row][col], ch) };
        }
    }
}
