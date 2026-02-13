set arch i386:x86-64
set print demangle on
set print asm-demangle on
set complaints 0

define rc
  file target/x86_64-kleinos/debug/kleinos
  hb kernel_main
  target remote :1234
  c
end
document rc
  Reload binary, reconnect to QEMU, and break at kernel_main.
end
