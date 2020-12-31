# 硬件

## 硬件

现在，你应该开始熟悉所使用的工具和开发过程。这一节，我们会切换到真实的硬件，过程大致相同，让我们开始吧。

## 了解你的硬件

在我们开始之前，需要确定用于这个项目的硬件设备的一些特征

-   ARM内核，例如`Cortex-m3`架构
-   ARM内核是否包含FPU？例如`Cortex-M4F`和`Cortex-M7F`包含FPU
-   硬件设备的`Flash`和`RAM`的大小？例如256KB的`Flash`和32KB的`RAM`
-   `Flash`和`RAM`地址空间的映射？例如RAM通常位于地址`0x2000_0000`

可以在设备的数据手册或者参考手册上找到这些信息



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

现在，可以使用`cargo build`交叉编译这个程序然后就像以前那样使用`cargo-binutils`检查二进制文件。`cortex-m-rt`crate处理所有让芯片的运行起来要求，有帮助的是，几乎所有`Cortex-M`的芯片都是以这种方式启动的

## 调试

程序的调试会看起来有些不一样。实际上，根据目标设备不同，第一步可能会有所不同。这一节，我们会展示调试跑在`STM32H750 ART-Pi`开发板的程序所需要的步骤。仅作为参考，关于设备设备调试的具体信息，请查阅[theDebugonomicon](https://github.com/rust-embedded/debugonomicon)。



像之前一样，我们会进行远程调试，客户端会是一个GDB进程。而这一次的服务器会是`OpenOCD`



按照验证部分做的一样，我们把开发板连接到电脑上，然后检查能否识别到`ST-LINK`



接下来，在一个终端运行`openocd`去连接开发板上的`ST-LINK`。在模板的跟目录下执行这条命令，`openocd`会读取配置文件`openocd`，指明要用到的接口文件和目标文件

```bash
cat openocd.cfg
```

```c
# Revision C (newer revision)
source [find interface/stlink-v2-1.cfg]

# Revision A and B (older revisions)
# source [find interface/stlink-v2.cfg]
source [find target/stm32h7x.cfg]
```

```bash
Open On-Chip Debugger 0.10.0+dev-01514-ga8edbd020-dirty (2020-12-26-11:43)
Licensed under GNU GPL v2
For bug reports, read
        http://openocd.org/doc/doxygen/bugs.html
WARNING: interface/stlink-v2-1.cfg is deprecated, please switch to interface/stlink.cfg
Info : auto-selecting first available session transport "hla_swd". To override use 'transport select <transport>'.
Info : The selected transport took over low-level target control. The results might differ compared to plain JTAG/SWD
Info : Listening on port 6666 for tcl connections
Info : Listening on port 4444 for telnet connections
Info : clock speed 1800 kHz
Info : STLINK V2J35M26 (API v2) VID:PID 0483:374B
Info : Target voltage: 3.246032
Info : stm32h7x.cpu0: hardware has 8 breakpoints, 4 watchpoints
Info : starting gdb server for stm32h7x.cpu0 on 3333
Info : Listening on port 3333 for gdb connections
```

在另一个终端运行`arm-none-eabi-gdb`，同样在根目录运行，`GDB`会读取目录下的`.gdbinit`文件

```c
file ./target/thumbv7em-none-eabihf/debug/hello
target extended-remote :3333
monitor reset halt
load
```

现在我们分开一步一步来，目标设备是`arm`的开发板，所以我们运行`arm-none-eabi-gdb`

第一步，导入一个二进制文件

```c
file ./target/thumbv7em-none-eabihf/debug/hello
```

然后连接`GDB`到`OpenOCD`，`openocd`正在等待`3333`端口的`TCP`连接

```c
(gdb) target extended-remote :3333
Remote debugging using :3333
0x08002d36 in ?? ()
```

最后使用`load`命令烧录程序到微控制器

```c
(gdb) load
Loading section .vector_table, size 0x400 lma 0x8000000
Loading section .text, size 0x1114 lma 0x8000400
Loading section .rodata, size 0x354 lma 0x8001514
Start address 0x8000400, load size 6248
Transfer rate: 5 KB/sec, 2082 bytes/write.
```

现在程序已经被烧录到开发板。这个程序使用半主机模式，所以在我们进行任何半主机的调用时，我们得告诉`OpenOCD`使能半主机模式。可以发送命令给`OpenOCD`来使用`monitor`命令

```c
(gdb) monitor arm semihosting enable
semihosting is enabled
```

>   可以调用`monitor help`命令来查看全部OPenOCD命令

像以前一样，我们可以使用断点和`continue`命令直接跳转到主函数

```c
(gdb) break main
Breakpoint 1 at 0x8000490: file src/main.rs, line 10.
(gdb) continue
Continuing.
Note: automatically using hardware breakpoints for read-only addresses.

Breakpoint 1, main () at src/main.rs:10
10      #[entry]
```

使用`next`指令执行下一步，应该回产生和以前同样的结果

```
(gdb)next
halted: PC: 0x08000496
```

此时应该会看到 "Hello, world!" 还有其他东西打印在OpenODC的控制台上

```c
Info : halted: PC: 0x08000496
Hello, world!
```



# 内存映射寄存器

嵌入式操作系统到目前为止只能通过正常的Rust代码和在RAM中移动数据得到。如果我们想获取任何进入或输出我们的系统的信息（例如让LED闪烁，检测按键按下或者在某种总线上和片外外设通讯），我们不得不进入外围设备的世界和他们的 “内存映射寄存器”



你会发现，需要去访问微控制器上外设的代码已经写好了，都在下面几个层次之中

![](https://docs.rust-embedded.org/book/assets/crates.png)

-   Mirco-architeture Crate - 这种 crate 处理任何微控制器所使用的处理器核心的通用历程，以及任何使用这种处理器核心类型的微控制器的所有通用的外设。例如，`cortex-m crate` 提供函数去使能或者禁用中断，对于所有基于`Cortex-M`的微控制器都是一样的。它还可以访问所有基于`Cotex-M`的微控制器的 'SysTick' 外设。
-   Peripheral Access Crate (PAC) - 这种 crate 是各种定义了你正在使用的特定型号的微控制器的内存包装器的轻微包装。例如，tm4c123x 用于德州仪器 Tiva-C TM4C123系列，或者 stm32f30x 用于意法半导体的 STM32F30x 系列。可以直接与寄存器进行交互，遵循微控制器参考手册中给出的每种外设的操作说明。
-   HAL Crate - 这些 crate 提供对使用者更友好的 API 对于特定处理器，一般是通过实现一些在 embedded-hal 定义的通用的 traits。例如，此 crate 可能会提供一个 带有构造函数的`Serial`结构体，构造函数接受 GPIO 引脚合适的设置和波特率，还提供一些 `write_byte`函数用于发送数据。有关 embedded-hal 的更多信息，详见“可移植性”一章。
-   Board Crate - 这些 crate 比 HAL Crate 多做了一步，通过预配置不同外设和GPIO引脚以适应特定的开发者套件或你正在使用的开发板，例如用于 STM32F3DISCOVERY开发板的 stm32f3-discovery。