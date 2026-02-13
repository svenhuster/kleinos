#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::{panic::PanicInfo, ptr::read_volatile};
use kleinos::{
    qemu::{QemuExitCode, qemu_exit},
    serial_print, serial_println,
};
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");
    kleinos::gdt::init();
    init_test_idt();

    stack_overflow();

    panic!("Execution continued after stack overflow");
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    qemu_exit(QemuExitCode::Success);
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(kleinos::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    let _: u8 = unsafe { read_volatile(0xb8000 as *const u8) };
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kleinos::test_panic_handler(info)
}
