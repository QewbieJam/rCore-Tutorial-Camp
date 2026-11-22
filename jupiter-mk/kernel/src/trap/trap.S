    .altmacro
    .macro SAVE_GP n
        sd x\n, \n*8(sp)
    .endm
    .macro LOAD_GP n
        ld x\n, \n*8(sp)
    .endm

    .section .text
    .globl __alltraps
    .globl __restore
    .globl __task_entry
    .align 2

# Trap entry from U-mode.
#   sscratch = top of current task's kernel stack
#   sp       = user stack pointer
__alltraps:
    csrrw sp, sscratch, sp      # sp = kstack_top, sscratch = user sp
    addi  sp, sp, -34*8         # allocate TrapContext

    sd x1, 1*8(sp)
    # x2 (sp) is saved below from sscratch.
    sd x3, 3*8(sp)
    sd x4, 4*8(sp)
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr

    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)

    csrr t2, sscratch            # user sp
    sd t2, 2*8(sp)

    mv a0, sp
    call trap_handler

    # trap_handler is `-> !`; be defensive.
1:  j 1b

# First entry of a task: TaskContext.sp = kstack_top, TrapContext is just
# below it.
__task_entry:
    addi a0, sp, -34*8
    j __restore

# a0 = *TrapContext
__restore:
    mv sp, a0

    # Next trap from this task must land at the same kstack top.
    addi t0, sp, 34*8
    csrw sscratch, t0

    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1

    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    ld x4, 4*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr

    ld sp, 2*8(sp)              # user stack pointer
    sret
