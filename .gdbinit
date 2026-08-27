set architecture aarch64
set endian little
set pagination off
set confirm off
set print pretty on
set disassemble-next-line on
set breakpoint pending on

file target/aarch64-kernel/debug/palmeto
directory init pine drivers shared

target remote :1234
break _start