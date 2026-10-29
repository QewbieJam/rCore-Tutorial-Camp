# Ripes 实验：为什么二元运算必须压栈
        li   sp, 0x8000
        li   a0, 42
        addi sp, sp, -16
        sd   a0, 0(sp)
        li   a0, 7
        mv   a1, a0
        ld   a0, 0(sp)
        addi sp, sp, 16
        add   a0, a0, a1
