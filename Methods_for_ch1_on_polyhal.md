### 基于polyhal的rcore tutorial ch1

## 

1、引入polyhal后，会自动把\#[polyhal::arch_entry]后的函数作为入口函数：

```rust
//The entry point
#[polyhal::arch_entry]
fn main(hartid: usize) {
    if hartid != 0 {
        return;
    }
    println!("[kernel] Hello, world!");
    polyhal::shutdown();
}
```

2、polyhal以\#[polyhal::arch_interrupt]后的函数作为中断服务的入口，即使helloworld暂时不需要中断，也必须包含一个该空函数，否则polyhal无法找到interrupt入口会发生链接错误

```rust
/// kernel interrupt
#[polyhal::arch_interrupt]
fn kernel_interrupt(_ctx: &mut TrapFrame, _trap_type: TrapType) {

}
```

3、关于输出，可以直接调用polyhal::DebugConsole提供的接口：

```rust
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            DebugConsole::putchar(c);
        }
        Ok(())
    }
}
.......
#[inline]
pub fn puts(buffer: &[u8]) {
    // use the main uart if it exists.
    for i in buffer {
        DebugConsole::putchar(*i);
    }
}
```

4、需要在src下面添加四种架构的链接文件，并编写.cargo/config.toml来指定不同架构的链接文件和编译器：

```
[build]
target = "riscv64gc-unknown-none-elf"
# target = 'aarch64-unknown-none-softfloat'
# target = 'x86_64-unknown-none'
# target = 'loongarch64-unknown-none'

[target.riscv64gc-unknown-none-elf]
rustflags = [
    "-Clink-arg=-Tsrc/linker-riscv64.ld",
    "-Cforce-frame-pointers=yes",
    '--cfg=board="qemu"',
]

[target.x86_64-unknown-none]
rustflags = [
    "-Clink-arg=-Tsrc/linker-x86_64.ld",
    "-Cforce-frame-pointers=yes",
    '-Clink-arg=-no-pie',
    '--cfg=board="qemu"',
]

[target.aarch64-unknown-none-softfloat]
rustflags = [
    "-Clink-arg=-Tsrc/linker-aarch64.ld",
    "-Cforce-frame-pointers=yes",
    '--cfg=board="qemu"',
]

[target.loongarch64-unknown-none]
rustflags = [
    "-Clink-arg=-Tsrc/linker-loongarch64.ld",
    "-Cforce-frame-pointers=yes",
    '--cfg=board="qemu"',
]

```

5、需要修改rcore tutorial的makefile脚本来执行执行4种不同的架构：

```makefile
ARCH := riscv64
ifeq ($(ARCH), x86_64)
  TARGET := x86_64-unknown-none
  QEMU_EXEC += qemu-system-x86_64 \
				-machine q35 \
				-kernel $(KERNEL_ELF) \
				-cpu IvyBridge-v2
  BUS := pci
else ifeq ($(ARCH), riscv64)
  TARGET := riscv64gc-unknown-none-elf
  QEMU_EXEC += qemu-system-$(ARCH) \
				-machine virt \
				-kernel $(KERNEL_BIN)
else ifeq ($(ARCH), aarch64)
  TARGET := aarch64-unknown-none-softfloat
  QEMU_EXEC += qemu-system-$(ARCH) \
				-cpu cortex-a72 \
				-machine virt \
				-kernel $(KERNEL_BIN)
else ifeq ($(ARCH), loongarch64)
  TARGET := loongarch64-unknown-none
  QEMU_EXEC += qemu-system-$(ARCH) -kernel $(KERNEL_ELF)
  BUS := pci
else
  $(error "ARCH" must be one of "x86_64", "riscv64", "aarch64" or "loongarch64")
endif
```

