set history save on
set confirm off
set remotetimeout 240
target extended-remote :2331
set print asm-demangle on
monitor reset halt
load
tbreak main
continue
# quit
