/* linker.ld —— 把目标文件摆到 QEMU virt 认识的地址上
 *
 * 核心只有三件事：
 *   1. 基址 0x80200000（OpenSBI 之后 S-mode 能用的开始）
 *   2. 入口符号 _start（ELF 头的 e_entry）
 *   3. 段顺序：.text.entry 排最前，栈放 .bss 末尾
 */

OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    /* QEMU virt: 0x80000000 起是 DRAM，前 2MiB 留给 OpenSBI */
    . = 0x80200000;

    /* .text.entry 必须第一个：保证 _start 就位于镜像开头 */
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    .rodata ALIGN(16) : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    .data ALIGN(16) : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    /* 栈放在 .bss 末尾：不需要从文件加载，但 memsz 里要占住 */
    .bss ALIGN(16) : {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)

        . = ALIGN(16);
        _stack_bottom = .;
        . += 0x10000;           /* 64 KiB 栈 */
        _stack_top = .;
    }
}
