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

qemu for running it, otherwise none (probably)

## Diagram

too lazy
