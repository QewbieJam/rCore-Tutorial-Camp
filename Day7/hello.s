/* hello.s —— 脚手架自检：不经过编译器，直接手写一个 main
 * 用途：先确认 runtime.S / linker.ld / build.sh / qemu 全通，
 *       再动手写编译器，避免"到底是编译器错还是脚手架错"。
 *
 * 注意：main 里调了函数 -> main 就是"非叶子函数" -> 必须保存 ra！
 */

    .section .text
    .globl main
main:
    addi    sp, sp, -16             /* 非叶子函数：开帧存 ra */
    sd      ra, 8(sp)

    la      a0, msg
    call    puts_z
    call    print_nl

    li      a0, 42
    call    print                   /* 打印 "42" + 换行 */

    li      a0, -7
    call    print                   /* 负数测试 */

    li      a0, 0
    ld      ra, 8(sp)               /* 恢复 ra，否则 ret 回到最后一次 call 之后 */
    addi    sp, sp, 16
    ret

    .section .rodata
msg:
    .asciz "scaffold OK"
