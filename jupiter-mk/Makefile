BOARD ?= qemu
MODE ?= release
TARGET := riscv64gc-unknown-none-elf
KERNEL_ELF := target/$(TARGET)/$(MODE)/jupiter-kernel

ifeq ($(BOARD),jupiter)
  FEATURES := --no-default-features --features board_jupiter
else
  FEATURES := --features board_qemu
endif

QEMU := qemu-system-riscv64
QEMU_ARGS := -machine virt -nographic -bios default -m 128M

.PHONY: all build user kernel run run-jupiter sd-bundle debug clean fmt

all: build

# User programs are embedded into the kernel image at compile time, so they
# must be built first.
user:
	cd user && cargo build --release

kernel: user
	BOARD=$(BOARD) cargo build $(FEATURES) --$(MODE)

build: kernel

run: build
	$(QEMU) $(QEMU_ARGS) -kernel $(KERNEL_ELF)

run-jupiter: kernel
	@echo "Jupiter kernel: $(KERNEL_ELF)"
	@echo "Load it with U-Boot; see docs/jupiter-boot.md."

# 准备 SD 卡启动包：ELF + U-Boot 脚本
sd-bundle: kernel
	@mkdir -p sd-boot
	cp $(KERNEL_ELF) sd-boot/jupiter-kernel
	mkimage -A riscv -O linux -T script -C none -d sd-boot/boot.cmd sd-boot/boot.scr

debug: build
	$(QEMU) $(QEMU_ARGS) -kernel $(KERNEL_ELF) -s -S

clean:
	cargo clean
	cd user && cargo clean

fmt:
	cargo fmt --all
	cd user && cargo fmt --all
