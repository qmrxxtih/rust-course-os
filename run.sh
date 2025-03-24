#!/bin/bash

# Small build script to build it all.

# Check for requirements
requirements=("qemu-system-x86_64" "grub-mkrescue" "cargo" "nasm" "xorriso")

for req in "${requirements[@]}"; do
    if ! command -v "${req}" > /dev/null; then
        echo "missing dependency, please check you have all of following commands available : ${requirements[*]}"
        exit 1
    fi
done

# BUILD IT ALL

# Create build directory if it does not exist
[ ! -d ./build ] && mkdir -p ./build

# Build the Rust part
echo "BUILDING RUST BINARY"
cargo build --release --target x86_64-unknown-none

# Build the assemblies
echo "BUILDING ASSEMBLIES"
for x in $(ls asm/*.S); do
  outname=$(basename ${x})
  nasm -f elf64 -o build/${outname%%S}o ${x} 
done

# Link it together
echo "LINKING"
#ld -n -o build/image.bin -T linker/linker.ld build/*.o target/x86_64-unknown-none/release/libmink.a
ld -n -o build/image.bin -T linker/link.ld build/*.o target/x86_64-unknown-none/release/libmink.a

# Move the linked binary into ISO structure
echo "BUILDING ISO"
cp -f build/image.bin iso/boot/image.bin

# Build the ISO
grub-mkrescue -d /usr/lib/grub/i386-pc -o mink.iso iso/

# Boot the ISO
qemu-system-x86_64 -cdrom mink.iso
