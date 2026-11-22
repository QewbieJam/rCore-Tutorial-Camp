    .section .text.entry
    .globl _start
_start:
    # OpenSBI/U-Boot may pass hartid in a0 and the DTB physical address in a1.
    # Keep both intact while setting up the boot stack.
    la sp, boot_stack_top

    # Zero .bss. Some boot paths (raw `go` of an objcopy image) do not zero
    # NOBITS sections, so the kernel cannot rely on the loader here.
    la t0, sbss
    la t1, ebss
    bgeu t0, t1, 2f
1:
    sd zero, 0(t0)
    addi t0, t0, 8
    bltu t0, t1, 1b
2:
    fence rw, rw
    fence.i

    call rust_main

3:
    j 3b

    .section .bss.stack
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 65536
    .globl boot_stack_top
boot_stack_top:
