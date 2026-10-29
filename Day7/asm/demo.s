# rustc -C opt-level=0 -C debug-assertions=off 生成的 add3 程序（去掉伪指令、补入口）
        jal  ra, main               # 0x04
add3:
        add  a0, a0, a1             # 0x08
        add  a0, a0, a2             # 0x0c
        ret                         # 0x10                                                                                  
main:
        addi sp, sp, -16            # 0x14
        sd   ra, 8(sp)              # 0x18
        li   a0, 1                  # 0x1c
        li   a1, 2                  # 0x20
        li   a2, 3                  # 0x24
        jal  ra, add3               # 0x28
        ld   ra, 8(sp)              # 0x2c
        addi sp, sp, 16             # 0x30 释放栈帧 / 弹出帧,回到调用者调用时的 sp
        j    .
# 最后一句死循环，a0为6，ra指向0x8

