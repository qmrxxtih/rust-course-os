.POSIX:

TARGET = debug
OUTDIR = out

NASM ?= nasm
LD ?= ld
CARGO ?= cargo

CARGO_FLAGS_RELEASE = --release
CARGO_FLAGS_DEBUG =
CARGO_FLAGS =
EXTRA_CARGO_FLAGS =
LD_FLAGS = -n -T linker/link.ld

ifeq (${TARGET}, debug)
	CARGO_FLAGS += ${CARGO_FLAGS_DEBUG}
else ifeq (${TARGET}, release)
	CARGO_FLAGS += ${CARGO_FLAGS_RELEASE}
else
	ERR = $(error invalid TARGET '${TARGET}')
endif

SOURCES = $(wildcard src/*.rs)
ASM = $(wildcard boot/*.S)
OBJECTS = $(patsubst boot/%.S,${OUTDIR}/${TARGET}/obj/%.o,${ASM})

ISO = ${OUTDIR}/${TARGET}/mink.iso
IMAGE = ${OUTDIR}/${TARGET}/iso/boot/image.bin
LIB = ${OUTDIR}/custom_target/${TARGET}/libmink.a

.SUFFIXES:
.PHONY: all clean run

.DEFAULT_GOAL = all

run: all
	qemu-system-x86_64 -cdrom ${ISO}

all: ${ISO}

${ISO}: ${IMAGE} boot/grub.cfg
	@mkdir -p $(dir $<)/grub
	cp -f boot/grub.cfg $(dir $<)/grub/
	grub-mkrescue -d /usr/lib/grub/i386-pc -o $@ ${OUTDIR}/${TARGET}/iso

${IMAGE}: linker/link.ld ${OBJECTS} ${LIB}
	@mkdir -p $(dir $@)
	${LD} ${LD_FLAGS} ${OBJECTS} ${LIB} -o $@

${LIB}: ${SOURCES}
	@mkdir -p $(dir $@)
	${CARGO} +nightly build ${CARGO_FLAGS} ${EXTRA_CARGO_FLAGS} --target-dir ${OUTDIR}

${OUTDIR}/${TARGET}/obj/%.o: boot/%.S
	@mkdir -p $(dir $@)
	${NASM} -f elf64 -o $@ $<

clean:
	-rm -rf ${OUTDIR}
