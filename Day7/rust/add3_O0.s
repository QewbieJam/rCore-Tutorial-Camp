# rustc -C opt-level=0 -C debug-assertions=off 版本（去伪指令、补栈和入口）
        li   sp, 0x8000
        jal  ra, main
add3:
        add  a0, a0, a1
        add  a0, a0, a2
        ret
main:
        addi sp, sp, -16
        sd   ra, 8(sp)
        li   a0, 1
        li   a1, 2
        li   a2, 3
        jal  ra, add3
        ld   ra, 8(sp)
        addi sp, sp, 16
        j    .
