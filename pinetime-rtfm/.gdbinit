target remote :3333

set backtrace limit 32

#set arm force-mode thumb
#monitor arm semihosting enable

load
break main
continue
