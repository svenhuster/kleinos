# KleinOS

A bare-metal x86_64 toy kernel written in Rust.

Follows and draws heavily from Philipp Oppermann's [Writing an OS in Rust](https://os.phil-opp.com/) (blog_os) series.

## Resources

### Rust
- [Writing an OS in Rust](https://os.phil-opp.com/)
- [Rustonomicon](https://doc.rust-lang.org/nomicon/) — data layout, Send/Sync, unsafe contracts
- [Rust Inline Assembly](https://doc.rust-lang.org/reference/inline-assembly.html)
- [`core` Library Docs](https://doc.rust-lang.org/core/)
- [`x86_64` Crate Docs](https://docs.rs/x86_64/latest/x86_64/) — structures for IDT, GDT, page tables, port I/O
- [Tracking issue: `x86-interrupt` calling convention](https://github.com/rust-lang/rust/issues/40180) — `abi_x86_interrupt` feature gate

### OSDev
- [OSDev Wiki](https://wiki.osdev.org/Expanded_Main_Page)
  - [IDT](https://wiki.osdev.org/IDT)
  - [Exceptions](https://wiki.osdev.org/Exceptions)
  - [8259 PIC](https://wiki.osdev.org/8259_PIC)
  - [Paging](https://wiki.osdev.org/Paging)
  - [Page Frame Allocation](https://wiki.osdev.org/Page_Frame_Allocation)
  - [PS/2 Keyboard](https://wiki.osdev.org/PS/2_Keyboard)
  - [PIT](https://wiki.osdev.org/Programmable_Interval_Timer)
  - [Memory Map](https://wiki.osdev.org/Memory_Map_(x86))
  - [BIOS](https://wiki.osdev.org/BIOS)
- [Ralf Brown's Interrupt List](http://www.ctyme.com/rbrown.htm) — BIOS interrupt/service reference
- [OSDev Forums](https://forum.osdev.org/)

### Debugging
- [QEMU GDB Stub](https://qemu-project.gitlab.io/qemu/system/gdb.html) — `info registers`, `info mem`, `info tlb`
- [OSDev Wiki: Kernel Debugging](https://wiki.osdev.org/Kernel_Debugging)
- `rust-gdb` — GDB with Rust pretty-printers; can inspect statics from debug builds (e.g. `print 'kleinos::vga::SCREEN'`)

### Hardware References
- [Intel Software Developer Manuals](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html) — Vol 3A for IDT, paging, GDT/TSS, control registers; Vol 2A-2D for instruction set
- [AMD Architecture Programmer's Manual](https://docs.amd.com/v/u/en-US/24593_3.43) — Vol 2 System Programming; sometimes clearer than Intel on paging
- [System V ABI (x86_64)](https://gitlab.com/x86-psABIs/x86-64-ABI) — calling conventions, stack frame layout, register usage
- [8259 PIC Datasheet](https://pdos.csail.mit.edu/6.828/2005/readings/hardware/8259A.pdf)
- [UART 8250/16550 Programming](https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming)
- [ACPI Specification](https://uefi.org/specifications)

### Books
- [OSTEP](https://pages.cs.wisc.edu/~remzi/OSTEP/) — free; virtualization chapters (13-24) most relevant
- Modern Operating Systems (Tanenbaum, 5th ed, ISBN 978-0137618842)
- Operating System Concepts (Silberschatz, 10th ed, ISBN 978-1119800361)
- Computer Systems: A Programmer's Perspective (Bryant & O'Hallaron, 3rd ed, ISBN 978-0134092669)
