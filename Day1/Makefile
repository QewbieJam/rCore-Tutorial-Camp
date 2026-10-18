# SPDX-License-Identifier: GPL-2.0
# 树外 Rust 内核模块。
#
# KDIR 必须指向完整编译过、且 CONFIG_RUST=y 的内核源码树。

KDIR ?= /lib/modules/$(shell uname -r)/build

obj-m := rust_hello.o

all:
	$(MAKE) -C $(KDIR) M=$(CURDIR) modules

clean:
	$(MAKE) -C $(KDIR) M=$(CURDIR) clean

.PHONY: all clean
