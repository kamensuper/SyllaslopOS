.DEFAULT_GOAL := all
.DELETE_ON_ERROR:

BUILD := build
RUSTC ?= rustc
GRUB_FILE ?= grub-file
GRUB_MKRESCUE ?= grub-mkrescue
GRUB_PLATFORM ?= /usr/lib/grub/i386-pc
GRUB_DEFAULT ?= console
QEMU_DISPLAY ?= gtk
KERNEL_INPUTS := $(wildcard kernel/*.rs kernel/console/*.rs kernel/graphics/*.rs kernel/graphics/assets/*.outline)
QEMU ?= qemu-system-x86_64
QEMU_BIOS ?=
QEMU_MEMORY ?= 128M
PYTHON ?= python3
FAULTS := null ud de gp stack

ISO := $(BUILD)/syllaslopos.iso
KERNEL := $(BUILD)/kernel.elf
IMAGE_DATE := 2025010100000000
QEMU_FLAGS := -machine pc-i440fx-7.2 -accel tcg -cpu qemu64 -m $(QEMU_MEMORY) \
	$(if $(QEMU_BIOS),-bios "$(QEMU_BIOS)") -boot d -cdrom "$(ISO)" -nic none \
	-no-reboot
QEMU_HEADLESS := -display none -monitor none -serial stdio

.PHONY: all check test test-host test-console test-faults run run-serial debug clean
all: $(ISO)

$(BUILD):
	mkdir -p $@

$(BUILD)/boot.o: arch/x86_64/boot.S Makefile | $(BUILD)
	$(AS) --64 -o $@ $<

$(BUILD)/libkernel.a: kernel/main.rs $(KERNEL_INPUTS) rust-toolchain.toml Makefile | $(BUILD)
	$(RUSTC) --edition=2021 --crate-name syllaslopos --crate-type staticlib \
		--target x86_64-unknown-none -C code-model=kernel -C relocation-model=static \
		-C opt-level=2 -C panic=abort -C codegen-units=1 -o $@ $<

$(KERNEL): $(BUILD)/boot.o $(BUILD)/libkernel.a arch/x86_64/linker.ld Makefile
	$(LD) -m elf_x86_64 -T arch/x86_64/linker.ld --gc-sections --no-undefined \
		--build-id=none --strip-debug --fatal-warnings -z max-page-size=0x1000 -o $@ $(filter %.o %.a,$^)
	$(GRUB_FILE) --is-x86-multiboot2 $@

$(ISO): $(KERNEL) boot/grub/grub.cfg Makefile
	@set -eu; stage=$$(mktemp -d "$(BUILD)/iso.XXXXXX"); \
		trap 'rm -rf "$$stage"' EXIT; \
		mkdir -p "$$stage/root/boot/grub"; \
		cp "$(KERNEL)" "$$stage/root/boot/kernel.elf"; \
		case "$(GRUB_DEFAULT)" in console|compact|serial) ;; *) exit 2;; esac; \
		sed 's/@GRUB_DEFAULT@/$(GRUB_DEFAULT)/' boot/grub/grub.cfg > "$$stage/root/boot/grub/grub.cfg"; \
		LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=1735689600 $(GRUB_MKRESCUE) \
			-d "$(GRUB_PLATFORM)" -o "$$stage/boot.iso" "$$stage/root" \
			-volid SYLLASLOPOS --modification-date=$(IMAGE_DATE) --set_all_file_dates $(IMAGE_DATE); \
		mv "$$stage/boot.iso" "$@"

check: $(KERNEL)
	$(GRUB_FILE) --is-x86-multiboot2 "$(KERNEL)"
	readelf -h -lW "$(KERNEL)"

test: check $(ISO)
	sha256sum "$(KERNEL)" "$(ISO)"
	$(PYTHON) tools/boot-check.py "$(BUILD)/boot.log" -- $(QEMU) $(QEMU_FLAGS) $(QEMU_HEADLESS)

$(BUILD)/host-tests: kernel/host_tests.rs $(KERNEL_INPUTS) rust-toolchain.toml Makefile | $(BUILD)
	$(RUSTC) --edition=2021 --test -o $@ $<

test-host: $(BUILD)/host-tests
	"$<"

test-console: $(ISO) test-host
	$(MAKE) BUILD="$(BUILD)/compact" GRUB_DEFAULT=compact all
	$(MAKE) BUILD="$(BUILD)/recovery" GRUB_DEFAULT=serial all
	$(PYTHON) tools/console-check.py --output "$(BUILD)/screens" --iso "$(ISO)" \
		--resolution 1024x768x32 --qemu $(QEMU) $(if $(QEMU_BIOS),-bios "$(QEMU_BIOS)")
	$(PYTHON) tools/console-check.py --output "$(BUILD)/screens-compact" --iso "$(BUILD)/compact/syllaslopos.iso" \
		--resolution 800x600x32 --qemu $(QEMU) $(if $(QEMU_BIOS),-bios "$(QEMU_BIOS)")
	$(PYTHON) tools/console-check.py --output "$(BUILD)/screens-recovery" --iso "$(BUILD)/recovery/syllaslopos.iso" \
		--qemu $(QEMU) $(if $(QEMU_BIOS),-bios "$(QEMU_BIOS)")

# Each fatal fault halts the guest, so every kind gets its own boot. The
# recovery ISO checks the COM1 report; the default ISO checks the crash screen.
test-faults: $(ISO)
	$(MAKE) BUILD="$(BUILD)/recovery" GRUB_DEFAULT=serial all
	@set -e; for kind in $(FAULTS); do \
		$(PYTHON) tools/console-check.py --output "$(BUILD)/faults/$$kind" \
			--iso "$(BUILD)/recovery/syllaslopos.iso" --fault $$kind \
			--qemu $(QEMU) $(if $(QEMU_BIOS),-bios "$(QEMU_BIOS)"); \
	done
	$(PYTHON) tools/console-check.py --output "$(BUILD)/faults/screen" --iso "$(ISO)" \
		--resolution 1024x768x32 --fault null --qemu $(QEMU) $(if $(QEMU_BIOS),-bios "$(QEMU_BIOS)")

run: $(ISO)
	$(QEMU) $(QEMU_FLAGS) -display "$(QEMU_DISPLAY)" -monitor none -serial stdio

run-serial: $(ISO)
	$(QEMU) $(QEMU_FLAGS) $(QEMU_HEADLESS)

debug: $(ISO)
	$(QEMU) $(QEMU_FLAGS) $(QEMU_HEADLESS) -S -gdb tcp:127.0.0.1:1234

clean:
	rm -rf "$(BUILD)"
