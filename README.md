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

- boot custom code on x86 architecture
- a basic cli interface (ideally something similar to sh)
- basic device detection
- drivers for:
  - FAT32 filesystem
  - VGA
  - PS/2
  - serial io
- ability to write to filesystem
- executables
- allocator
- paging
- scheduler

## Dependencies

- Functional Rust environment (obviously)

- `NASM` for compiling the bootstrap code

- `QEMU` (mainly `qemu-system-x86_64`) for system emulation (requires x86-64 emulation to be possible)

- `GRUB` development package (`grub-mkrescue` has to be available)

- `xorriso` for creation of Multiboot image

- `make` for automated building and running

- In case they are not packed by default with GRUB, you also need GRUB BIOS files (folder `/usr/lib/grub/i386-pc/` must exist and not be empty)

## Running the code

To compile the code, run `make TARGET=release` to build in release mode. After that you can run the resulting image using `make run` command.

## Technical description

Our operating system uses [multiboot2](https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html) specification as its base. Multiboot allows booting of multiple operating systems using one unified standard specification. Most important part is the Multiboot header, which has fixed format and in our case is defined in `asm/multiboot.S` file. Multiboot launches selected OS image, sets up the CPU into valid protected mode, and handles control to our startup procedure (defined in `asm/boot.S` as `_start`), from which, theoretically, one could jump into Rust code and call it a day. However, modern Rust is built on x86_**64**, that is, 64-bit architecture, and protected mode only supports 32-bit instructions and constructs. So we need to set the CPU into so called "long mode". The setup is done using freely available code from [OSdev wiki](https://wiki.osdev.org/Setting_Up_Long_Mode). General order of swithing into long mode is:

- Check if CPU supports `CPUID` instruction for retrieval of CPU related information,

- Check if CPU supports long mode using the `CPUID` instruction

- Set up initial paging, since protected mode uses segmentation

- Jump to 64-bit bootstrap code (defined in `boot64.S`)


## Diagram

too lazy
