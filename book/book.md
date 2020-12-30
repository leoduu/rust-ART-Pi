# 硬件

## 硬件

现在，你应该开始熟悉所使用的工具和开发过程。这一节，我们会切换到真实的硬件，过程大致相同，让我们开始吧。

## 了解你的硬件

在我们开始之前，需要确定用于这个项目的硬件设备的一些特征

-   ARM内核，例如`Cortex-m3`架构
-   ARM内核是否包含FPU？例如`Cortex-M4F`和`Cortex-M7F`包含FPU
-   硬件设备的`Flash`和`RAM`的大小？例如256KB的`Flash`和32KB的`RAM`
-   `Flash`和`RAM`地址空间的映射？例如RAM通常位于地址`0x2000_0000`

你可以在设备的数据手册或者参考手册上找到这些信息



这一节，我们会用使用`ART-Pi`开发板作为参考，这个开发板包含一块`stm32H7XBH6`微控制器，有以下信息

-   Cortex-M7F内核，包含一个单精度的FPU
-   128KB的Flash，位于0x0800_0000
-   128KB的RAM，位于0x2000_0000（DTCM，还有ITCM和SRAM先忽略）

## 配置

我们从新建一个模板实例开始。参阅上一节，复习一下在没有`cargo-generate`应该怎么做

```bash
$ cargo generate --git https://github.com/rust-embedded/cortex-m-quickstart
 Project Name: app
 Creating project called `app`...
 Done! New project created /tmp/app
 
$ cd app
```

第一步，在`.cargo/config`设置默认编译目标

```bash
$ tail -n5 .cargo/config

# Pick ONE of these compilation targets
# target = "thumbv6m-none-eabi"    # Cortex-M0 and Cortex-M0+
# target = "thumbv7m-none-eabi"    # Cortex-M3
# target = "thumbv7em-none-eabi"   # Cortex-M4 and Cortex-M7 (no FPU)
target = "thumbv7em-none-eabihf" # Cortex-M4F and Cortex-M7F (with FPU)
```

我们会用`thumbv7em-none-eabifh`，因为它包含`Cortex-M7F`内核

第二步，把内存区域的信息写入`memory.x`文件

```c
$ cat memory.x
/* Linker script for the STM32H750XB */
MEMORY
{
  /* NOTE 1 K = 1 Kbytes = 1024 bytes */
  FLASH : ORIGIN = 0x08000000, LENGTH = 128K
  RAM : ORIGIN = 0x20000000, LENGTH = 128K 	 	// DTCM
}
```

确保函数`debug::exit()`被注释或者删除，这个函数只在qemu下起作用

```rust
#[entry]
fn main() -> ! {
    hprintln!("Hello, world!").unwrap();
    // exit QEMU
    // NOTE do not run this on hardware; it can corrupt OpenOCD state
    // debug::exit(debug::EXIT_SUCCESS);
    loop {}
}
```

现在，你可以使用`cargo build`交叉编译这个程序然后就像以前那样使用`cargo-binutils`检查二进制文件。`cortex-m-rt`crate处理所有让你的芯片的运行起来要求，对你有帮助的是，几乎所有`Cortex-M`的芯片都是以这种方式启动的

## 调试

程序的调试会看起来有些不一样。实际上，根据目标设备不同，第一步可能会有所不同。这一节，我们会展示调试跑在`STM32H750 ART-Pi`开发板的程序所需要的步骤。仅作为参考，关于设备设备调试的具体信息，请查阅[theDebugonomicon](https://github.com/rust-embedded/debugonomicon)。



像之前一样，我们会进行远程调试，客户端会是一个GDB进程。而这一次的服务器会是`OpenOCD`



按照验证部分做的一样，我们吧开发板连接到你的电脑上，然后检查能否识别到`ST-LINK`



接下来，在一个终端运行`openocd`去连接开发板上的`ST-LINK`。在模板的跟目录下执行这条命令，`openocd`会读取配置文件`openocd`，指明要用到的接口文件和目标文件

```bash
cat openocd.cfg
```

```c
# Sample OpenOCD configuration for the STM32F3DISCOVERY development board
# Depending on the hardware revision you got you'll have to pick ONE of these
# interfaces. At any time only one interface should be commented out.

# Revision C (newer revision)
source [find interface/stlink-v2-1.cfg]

# Revision A and B (older revisions)
# source [find interface/stlink-v2.cfg]
source [find target/stm32h7x.cfg]
```

```bash
openocd
Open On-Chip Debugger 0.10.0
Licensed under GNU GPL v2
For bug reports, read
        http://openocd.org/doc/doxygen/bugs.html
Info : auto-selecting first available session transport "hla_swd". To override use 'transport select <transport>'.
adapter speed: 1000 kHz
adapter_nsrst_delay: 100
Info : The selected transport took over low-level target control. The results might differ compared to plain JTAG/SWD
none separate
Info : Unable to match requested speed 1000 kHz, using 950 kHz
Info : Unable to match requested speed 1000 kHz, using 950 kHz
Info : clock speed 950 kHz
Info : STLINK v2 JTAG v27 API v2 SWIM v15 VID 0x0483 PID 0x374B
Info : using stlink api v2
Info : Target voltage: 2.913879
Info : stm32f3x.cpu: hardware has 6 breakpoints, 4 watchpoints
```

在另一个终端运行`GDB`，同样在根目录运行，`GDB`会读取目录下的`.gdbinit`文件

```c
file ./target/thumbv7em-none-eabihf/debug/blink
target extended-remote :3333
monitor reset halt
load
```

现在我们分开一步一步来，目标设备是`arm`的开发板，所以我们运行`arm-none-eabi-gdb`

第一步，导入一个二进制文件

```c
file ./target/thumbv7em-none-eabihf/debug/blink
```

然后连接`GDB`到`OpenOCD`，`openocd`正在等待`3333`端口的`TCP`连接

```c
(gdb) target remote :3333
Remote debugging using :3333
0x00000000 in ?? ()
```

最后使用`load`命令烧录程序到微控制器

```c
(gdb) load
Loading section .vector_table, size 0x400 lma 0x8000000
Loading section .text, size 0x1e70 lma 0x8000400
Loading section .rodata, size 0x61c lma 0x8002270
Start address 0x800144e, load size 10380
Transfer rate: 17 KB/sec, 3460 bytes/write.
```

现在程序已经被烧录到开发板。这个程序使用半主机模式，所以在我们进行任何半主机的调用时，我们得告诉`OpenOCD`使能半主机模式。你可以发送命令给`OpenOCD`来使用`monitor`命令

```c
(gdb) monitor arm semihosting enable
semihosting is enabled
```

>   You can see all the OpenOCD commands by invoking the `monitor help` command.

