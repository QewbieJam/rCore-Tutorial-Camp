/* runtime.S —— 裸机运行时（RISC-V 64, SysV ABI, SBI 输出）
 *
 * 编译器生成的代码只要 call 这几个符号即可：
 *   putchar(c)      输出 1 个字符
 *   puts_z(s)       输出 0 结尾字符串
 *   print_int(x)    输出整数（支持负数）
 *   print_nl()      换行
 *   print(x)        整数 + 换行
 *
 * 约定：a0 传参 / 返回；s0、ra 由被调者保存；sp 16 字节对齐。
 */

    .section .text.entry
    .globl _start
_start:
    la      sp, _stack_top          /* 立栈：之后所有 sd/ld 才有地方去 */
    call    main                    /* 用户程序的入口 */
    /* 关机：legacy SBI_SHUTDOWN（a7=8）。注意新版 SRST(0x53525354)
     * 在 QEMU 7.0 + OpenSBI v1.0 上返回 -2 (NOT_SUPPORTED)，别用 */
    li      a7, 8
    ecall
1:  j       1b                      /* 万一没关掉，别跑飞 */

/* ------------------------------------------------------------------ */
/* void putchar(long c)   a0 = 字符                                    */
    .globl putchar
putchar:
    li      a7, 1                   /* SBI legacy console_putchar */
    ecall
    ret

/* ------------------------------------------------------------------ */
/* void puts_z(const char *s)   a0 = 以 0 结尾的字符串                 */
    .globl puts_z
puts_z:
    addi    sp, sp, -16
    sd      ra, 8(sp)
    sd      s0, 0(sp)
    mv      s0, a0                  /* s0 当读指针（callee-saved，跨 call 安全） */
1:  lbu     a0, 0(s0)
    beqz    a0, 2f
    call    putchar
    addi    s0, s0, 1
    j       1b
2:  ld      ra, 8(sp)
    ld      s0, 0(sp)
    addi    sp, sp, 16
    ret

/* ------------------------------------------------------------------ */
/* void print_int(long x)   a0 = 整数（支持负数）                       */
/* 做法：从缓冲区尾部往前填数字，再整体打印                            */
    .globl print_int
print_int:
    addi    sp, sp, -48
    sd      ra, 40(sp)              /* sp+40 = s0-8  */
    sd      s0, 32(sp)              /* sp+32 = s0-16 */
    addi    s0, sp, 48              /* s0 = 帧底（高地址） */
    /* 帧布局：s0-8..s0-1 = 存 ra，s0-16..s0-9 = 存 s0
     * 缓冲区只能从 s0-17 往下长，不然会覆盖存好的 ra（踩过！） */
    addi    t0, s0, -17             /* 写指针：缓冲区顶端（避开寄存器槽） */
    sb      zero, 0(t0)             /* 先写字符串结束符 */
    mv      t1, a0
    li      t6, 0                   /* t6 = 负数标志 */
    bgez    t1, 1f
    li      t6, 1
    sub     t1, zero, t1            /* 取绝对值 */
1:  li      t4, 10
2:  remu    t5, t1, t4              /* t5 = x % 10 */
    divu    t1, t1, t4              /* t1 = x / 10 */
    addi    t5, t5, 48              /* '0' */
    addi    t0, t0, -1
    sb      t5, 0(t0)
    bnez    t1, 2b                  /* 还有高位就继续 */
    beqz    t6, 3f
    li      a0, 45                  /* '-' */
    call    putchar
3:  mv      a0, t0
    call    puts_z
    ld      ra, 40(sp)
    ld      s0, 32(sp)
    addi    sp, sp, 48
    ret

/* ------------------------------------------------------------------ */
/* void print_nl(void)                                                 */
    .globl print_nl
print_nl:
    li      a0, 10
    j       putchar                 /* 尾调用 */

/* ------------------------------------------------------------------ */
/* void print(long x)  —— 整数 + 换行                                   */
    .globl print
print:
    addi    sp, sp, -16
    sd      ra, 8(sp)
    call    print_int
    call    print_nl
    ld      ra, 8(sp)
    addi    sp, sp, 16
    ret
