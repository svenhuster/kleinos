use crate::x86_64::{inb, outb};

const COM1: u16 = 0x3f8;

#[repr(u16)]
enum Register {
    Data = 0,
    IntEn = 1,
    FifoCtrl = 2,
    LineCtrl = 3,
    ModemCtrl = 4,
    LineStatus = 5,
}

// SAFETY: SerialPort::new() creates a port handle for COM1 at 0x3F8,
// which is a valid fixed address. The Mutex ensures exclusive access.
pub static PORT: crate::Mutex<SerialPort> = crate::Mutex::new(unsafe { SerialPort::new() });

pub struct SerialPort {
    base: u16,
}

// SAFETY: SerialPort holds only a port address. I/O ports are global
// hardware resources accessible from any CPU context and are valid for the
// kernel lifetime.
unsafe impl Send for SerialPort {}

impl SerialPort {
    /// Creates a new SerialPort for COM1.
    ///
    /// # Safety
    ///
    /// The caller must ensure this is only used when the global `PORT` mutex
    /// cannot be used (e.g., in panic handlers to avoid deadlock). The port
    /// address 0x3F8 is always valid on x86, but `init()` must be called
    /// before writing to configure the UART.
    pub const unsafe fn new() -> Self {
        Self { base: COM1 }
    }

    fn port(&self, reg: Register) -> u16 {
        self.base + reg as u16
    }

    pub fn init(&mut self) {
        // TODO: add check if COM1 was detected at boot. Maybe init
        // should return a Result at that point. Possible improvement
        // to use the typestate pattern to ensure it's init before
        // use. Might require OnceLock to allow for a global static.

        // SAFETY: Port I/O to 0x3F8-0x3FD is well-defined on x86. Accessing
        // non-existent hardware returns 0xFF on reads and is ignored on writes; it
        // won't trigger DMA or corrupt memory.
        unsafe {
            outb(self.port(Register::IntEn), 0x00);
            outb(self.port(Register::LineCtrl), 0x80);
            outb(self.port(Register::Data), 0x01);
            outb(self.port(Register::IntEn), 0x00);
            outb(self.port(Register::LineCtrl), 0x03);
            outb(self.port(Register::FifoCtrl), 0xC7);
            outb(self.port(Register::ModemCtrl), 0x03);
        }
    }

    fn is_transmit_empty(&self) -> bool {
        // SAFETY: Reading LSR has no side-effects, is safe if COM1
        // exists and no other process will read or write outside of
        // the global static. Therefore, sequential access is guaranteed.
        unsafe { inb(self.port(Register::LineStatus)) & 0x20 != 0 }
    }

    pub fn write_byte(&mut self, byte: u8) {
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }
        // SAFETY: Writing to THR at the DATA port is valid and the
        // busy-wait on LSR ensures that it can accept data. No other
        // process will write to the port outside of the global
        // static, hence, the busy-wait guarantees sequential access.
        unsafe { outb(self.port(Register::Data), byte) };
    }
}

impl core::fmt::Write for SerialPort {
    // Only ASCII will be printed properly on the serial port
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

#[macro_export]
macro_rules! serial_print {
        ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
    }

#[macro_export]
macro_rules! serial_println {
        () => ($crate::serial_print!("\n"));
        ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
    }

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    PORT.lock().write_fmt(args).expect("serial write failed");
}
