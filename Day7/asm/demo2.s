#let x = (2 + 3) * 4
        li   sp, 0x8000
        li   a0, 2
        addi sp, sp, -16
        sd   a0, 0(sp)          # ← 内层 Add 的左值压栈
        li   a0, 3
        mv   a1, a0
        ld   a0, 0(sp)
        addi sp, sp, 16
        add  a0, a0, a1         # a0 = 5
        addi sp, sp, -16
        sd   a0, 0(sp)          # ← 外层 Mul 的左值压栈
        li   a0, 4
        mv   a1, a0
        ld   a0, 0(sp)
        addi sp, sp, 16
        mul  a0, a0, a1         # 5 * 4 = 20