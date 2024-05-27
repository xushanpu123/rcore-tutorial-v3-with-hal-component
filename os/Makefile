# Building
ARCH := riscv64
ifeq ($(ARCH), x86_64)
  TARGET := x86_64-unknown-none
  QEMU_EXEC += qemu-system-x86_64 \
				-serial mon:stdio \
				-machine q35 \
				-kernel $(KERNEL_ELF) \
				-cpu IvyBridge-v2 \
				-device virtio-gpu-pci -vga none \
				-device virtio-net-pci,netdev=net0 -netdev user,id=net0,hostfwd=tcp::5555-:5555
  BUS := pci
else ifeq ($(ARCH), riscv64)
  TARGET := riscv64gc-unknown-none-elf
  QEMU_EXEC += qemu-system-$(ARCH) \
				-machine virt \
				-device virtio-blk-device,drive=x0, \
				-kernel $(KERNEL_BIN)
else ifeq ($(ARCH), aarch64)
  TARGET := aarch64-unknown-none-softfloat
  QEMU_EXEC += qemu-system-$(AR	-serial mon:stdio \ -kernel $(KERNEL_ELF)
  BUS := pci
else
  $(error "ARCH" must be one of "x86_64", "riscv64", "aarch64" or "loongarch64")
endif
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm
FS_IMG := ../user/target/$(TARGET)/$(MODE)/fs.img
APPS := ../user/src/bin/*

# BOARD
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# GUI
GUI ?= off
ifeq ($(GUI), off)
	GUI_OPTION := -display none
endif

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80200000

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# Disassembly
DISASM ?= -x		
env:
	(rustup target list | grep "$(TARGET) (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

build: env $(KERNEL_BIN) fs-img 

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

fs-img: $(APPS)
	@cd ../user && make build TARGET=$(TARGET) TEST=$(TEST)
	@rm -f $(FS_IMG)
	@cargo install easyfs-packer && easyfs-packer -s ../user/src/bin/ -t ../user/target/$(TARGET)/release/
	cp ../user/target/$(TARGET)/$(MODE)/fs.img fs-img.img

$(APPS):

kernel:
	@echo Platform: $(BOARD)
	@cargo build -Z build-std --release --target $(TARGET)

clean:
	@cargo clean

run: run-inner

ifeq ($(ARCH), riscv64)
QEMU_EXEC += -drive file=$(FS_IMG),if=none,format=raw,id=x0
endif
QEMU_EXEC += -smp 1 \
			 -D qemu.log -d in_asm,int,pcall,cpu_reset,guest_errors \
			 $(GUI_OPTION) \
			 -device virtio-keyboard-pci \
			 -device virtio-mouse-pci \
			#  -serial stdio \
			#  -device virtio-gpu-device \
			#  -device virtio-net-device,netdev=net0 \
			#  -netdev user,id=net0,hostfwd=udp::6200-:2000,hostfwd=tcp::6201-:80


fdt:
	qemu-system-$(ARCH) -M 128m -machine virt,dumpdtb=virt.out
	fdtdump virt.out

run-inner: build
	$(QEMU_EXEC)

debug: build
	@tmux new-session -d \
		"$(QEMU_EXEC) -s -S" && \
		tmux split-window -h "gdb -ex 'file $(KERNEL_ELF)' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d


gdbserver: build
	@$(QEMU_EXEC) -s -	echo "exec"el clean disasm disasm-vim run-inner fs-img gdbserver gdbclient fdt