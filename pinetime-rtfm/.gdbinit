#target remote :3333
#set arm force-mode thumb
#monitor arm semihosting enable
#load
#step

target remote :2331
set backtrace limit 32
monitor semihosting enable
load
break main
