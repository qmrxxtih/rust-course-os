# MINK_OS

## names:

- Tomáš Čaplák
- Jakub Ozsvald
- Veronika Barchánková

## Intro

In this project, we aim to design and implement a microkernel operating system, with a focus on achieving a modular and highly extensible architecture. The core goal is to create a system where the kernel is kept minimal, with essential services running in user space. We hope this will improve system stability, security, and fault isolation compared to a monolithic approach.
Each component will operate independently and can be replaced or updated without affecting the core system.

By working on this project we want to learn low-level programming, nostd rust (which is also useful in embedded programming) and get a general grasp of an operating system's inner workings.

## Requirements

- [x] boot custom code on x86 architecture

- [x] a basic cli interface (ideally something similar to sh)

- [ ] basic device detection

- [ ] drivers for:

  - [ ] FAT32 filesystem

  - [x] VGA (builtin)

  - [x] PS/2 keyboard (builtin)

  - [ ] serial I/O

- [ ] ability to write to filesystem

- [ ] executables

- [ ] scheduler

- [x] allocator

- [x] paging

## Dependencies

- latest Rust `nightly` (you can use `rustup default nightly` to set this up automatically, if you have functional rust environment that is)

- `NASM` for compiling the bootstrap code

- `QEMU` (mainly `qemu-system-x86_64`) for system emulation (requires x86-64 emulation to be possible)

- `GRUB` development package (`grub-mkrescue` has to be available)

- `xorriso` for creation of Multiboot image

- `make` for automated building and running

- In case they are not packed by default with GRUB, you also need GRUB BIOS files (folder `/usr/lib/grub/i386-pc/` must exist and not be empty)

## Used packages

- `x86_64` - this crate provides simplified abstraction over some instructions and data structures required for work with x86_64 architecture.

- `linked_list_allocator` - provides simple heap allocation structure, representing free memory blocks (so called "holes") as entries in linked list.

- `lazy_static` - provides special kind of runtime "fake" static variables, used for mutable static-ish access on singleton structures.

- `spin` - provides very simple looping mutex lock guard.

- `pic8259` - provides simple interface over PIC8259 chip interface for interrupt initialisation.

## Running the code

To compile the code, run `make TARGET=release` to build in release mode. After that you can run the resulting image using `make run` command.

## Technical description

### Boot process

Our operating system uses [multiboot2](https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html) specification as its base. Multiboot allows booting of multiple operating systems using one unified standard specification. Most important part is the Multiboot header, which has fixed format and in our case is defined in `asm/multiboot.S` file. Multiboot launches selected OS image, sets up the CPU into valid protected mode, and handles control to our startup procedure (defined in `asm/boot.S` as `_start`), from which, theoretically, one could jump into Rust code and call it a day. However, modern Rust is built on x86\_**64**, that is, 64-bit architecture, and protected mode only supports 32-bit instructions and constructs. So we need to set the CPU into so called "long mode". The setup is done using freely available code from [OSdev wiki](https://wiki.osdev.org/Setting_Up_Long_Mode). General order of swithing into long mode is:

- Check if CPU supports `CPUID` instruction for retrieval of CPU related information,

- Check if CPU supports long mode using the `CPUID` instruction

- Set up initial paging, since protected mode uses segmentation

- Jump to 64-bit bootstrap code (defined in `boot64.S`)

- Launch the actual rust code

### Initial Rust steps

After the Rust code takes control, it will do few things initially - firstly, interrupts are initialised by loading interrupt routines into Interrupt Descriptor Table (IDT) data structure. This data structure is then passed to Control Register 2 by address reference, so the processor knows where it is located. After this, interrupt controllers are initialised and interrupts are enabled for the processor.

Next, heap memory is prepared, so we have access to dynamic memory structures, such as `Box`, `Vec`, and many more. This is done by firstly initialising special page frame allocator, providing it with memory map entries from multiboot, so it knows where it can put new page frames. Next, the OS maps special region at address `0x7000 0000 0000` with size of 1 MiB, and uses this region as new kernel heap. From now on, each dynamically allocated variable will reside here!

After these initial steps, few minor things are done, such as sending 2 bytes to VGA control registers to disable blinking cursor, and printing of the mink logo.

### Memory mapping

Memory is initially mapped by our small bootloader - initial memory for kernel is mapped into 512 huge pages (2 MiB in size, compared to standard page size of 4 KiB). This effectivelly gives our kernel very large memory space of 1 GiB, which is way more than enough for such simple kernel. The mapping is done 1 to 1, meaning that address 0 will indeed translate to address 0. This was deemed quite a simple, yet good enough solution for this project.

Another mapping happens when heap is initialised, as described in previous section. This mapping is placed on virtual address space far away from our kernel region. 1 MiB of heap memory is way more than enough for this project, since we will definitely not be running any memory intense applications (or any application in that matter).

Last kind of mapping is the one invoked manually - when required, one can simply call `paging::map_huge_mapper(...)` function to map new huge page (2 MiB in size) into the virtual address space, or `mapper.map_to(...)` method of `mapper` page mapper to map regular size page (4 KiB in size). Mapped memory can be freely used for anything, from extending kernel variable stack, to (in our case not possible to implement) inter-process communication.

### Interrupt handling

External hardware generates interrupts to let the processor know that something happened - keyboards generate interrupt every time key is pressed (or released), motherboard timers generate interrupt on every timer tick, and so on. To handle these interrupts, special data structure is used - Interrupt Descriptor Table, or IDT for short. This structure is pointed at by processor's CR2 register. Every time an interrupt is invoked by external hardware, processor pauses what it is currently doing, saves its state onto stack, reads interrupt request (IRQ) number from interrupt controller, and uses the number to index the IDT to get coresponding interrupt handler. This handler is then executed, and upon finishing signals the interrupt controller that interrupt was handled successfully. With that, interrupt controller waits for new interrupt, repeating the cycle.

Interrupts are sort of asynchronous operation - they can happen any moment and most of the time do not rely on processor's frequency clock. This means that in special cases, they can introduce dead locks - condition of locking access to shared resource, without re-unlocking it. This is treated by careful coding and use of interrupt disables whenever critical region is accessed, so another interrupt won't try to access currently locked resource.

### Shell

A basic shell interface implementation for our OS kernel, providing command-line functionality with input handling, command history, and multiple system commands. The shell supports user input through keyboard events, processes commands, and displays output via VGA text mode. Implemented features include command history using Up and Down arrows, line editing (backspace support), and several system commands like help, clear, echo, poweroff, and the multiboot command for system information.

The shell integrates with low-level system components, including keyboard input handling and VGA text output, and provides system control functions such as shutting down the machine via QEMU-specific ports or ACPI. It also parses and displays Multiboot2 bootloader information, including memory maps, loaded modules, and kernel details, using helper functions to format and print numeric values in decimal and hexadecimal. The clear_screen command includes a stylized OS logo, demonstrating basic ANSI-like color support through the VGA driver.

## Diagram

`unimplemented!()`
