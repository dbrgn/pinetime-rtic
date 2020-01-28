target remote :3333

set arm force-mode thumb

monitor arm semihosting enable

load
step
