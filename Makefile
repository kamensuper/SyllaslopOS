.DEFAULT_GOAL := all
.DELETE_ON_ERROR:

BUILD := build
RUSTC ?= rustc
GRUB_FILE ?= grub-file
GRUB_MKRESCUE ?= grub-mkrescue
GRUB_PLATFORM ?= /usr/lib/grub/i386-pc
QEMU ?= qemu-system-x86_64
QEMU_BIOS ?=
QEMU_MEMORY ?= 128M
PYTHON ?= python3

ISO := $(BUILD)/syllaslopos.iso
KERNEL := $(BUILD)/kernel.elf
IMAGE_DATE := 2025010100000000
QEMU_FLAGS := -machine pc-i440fx-7.2 -accel tcg -cpu qemu64 -m $(QEMU_MEMORY) \
	$(if $(QEMU_BIOS),-bios "$(QEMU_BIOS)") -boot d -cdrom "$(ISO)" -nic none \
	-no-reboot -display none -monitor none -serial stdio

.PHONY: all check test run debug clean
all: $(ISO)

$(BUILD):
	mkdir -p $@

$(BUILD)/boot.o: arch/x86_64/boot.S Makefile | $(BUILD)
	$(AS) --64 -o $@ $<

$(BUILD)/libkernel.a: kernel/main.rs kernel/serial.rs rust-toolchain.toml Makefile | $(BUILD)
	$(RUSTC) --edition=2021 --crate-name syllaslopos --crate-type staticlib \
		--target x86_64-unknown-none -C code-model=kernel -C relocation-model=static \
		-C opt-level=2 -C panic=abort -C codegen-units=1 -o $@ $<

$(KERNEL): $(BUILD)/boot.o $(BUILD)/libkernel.a arch/x86_64/linker.ld Makefile
	$(LD) -m elf_x86_64 -T arch/x86_64/linker.ld --gc-sections --no-undefined \
		--build-id=none --fatal-warnings -z max-page-size=0x1000 -o $@ $(filter %.o %.a,$^)
	$(GRUB_FILE) --is-x86-multiboot2 $@

$(ISO): $(KERNEL) boot/grub/grub.cfg Makefile
	@set -eu; stage=$$(mktemp -d "$(BUILD)/iso.XXXXXX"); \
		trap 'rm -rf "$$stage"' EXIT; \
		mkdir -p "$$stage/root/boot/grub"; \
		cp "$(KERNEL)" "$$stage/root/boot/kernel.elf"; \
		cp boot/grub/grub.cfg "$$stage/root/boot/grub/grub.cfg"; \
		LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=1735689600 $(GRUB_MKRESCUE) \
			-d "$(GRUB_PLATFORM)" -o "$$stage/boot.iso" "$$stage/root" \
			-volid SYLLASLOPOS --modification-date=$(IMAGE_DATE) --set_all_file_dates $(IMAGE_DATE); \
		mv "$$stage/boot.iso" "$@"

check: $(KERNEL)
	$(GRUB_FILE) --is-x86-multiboot2 "$(KERNEL)"
	readelf -h -lW "$(KERNEL)"

test: check $(ISO)
	sha256sum "$(KERNEL)" "$(ISO)"
	$(PYTHON) tools/boot-check.py "$(BUILD)/boot.log" -- $(QEMU) $(QEMU_FLAGS)

run: $(ISO)
	$(QEMU) $(QEMU_FLAGS)

debug: $(ISO)
	$(QEMU) $(QEMU_FLAGS) -S -gdb tcp:127.0.0.1:1234

clean:
	rm -rf "$(BUILD)"
