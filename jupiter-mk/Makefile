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

.PHONY: all build user kernel run run-jupiter debug clean fmt

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

debug: build
	$(QEMU) $(QEMU_ARGS) -kernel $(KERNEL_ELF) -s -S

clean:
	cargo clean
	cd user && cargo clean

fmt:
	cargo fmt --all
	cd user && cargo fmt --all
