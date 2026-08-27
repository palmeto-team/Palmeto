#
# Makefile 
#

KARCH         ?= aarch64
IMAGE_NAME    ?= palmeto
BUILD_PROFILE ?= dev
PROFILE_NAME  := $(if $(filter dev,$(BUILD_PROFILE)),debug,$(BUILD_PROFILE))

SCRIPT_DIR   := scripts
CHECK_DEPS   := $(SCRIPT_DIR)/check-deps.sh
INSTALL_DEPS := $(SCRIPT_DIR)/install-deps.sh

BUILD_DIR    := build
PROFILE_DIR  := $(BUILD_DIR)/$(PROFILE_NAME)
KERNEL_DIR   := pine
KERNEL_BIN   := target/aarch64-kernel/$(PROFILE_NAME)/$(IMAGE_NAME)
LIMINE_DIR   := $(BUILD_DIR)/limine
OVMF_DIR     := $(BUILD_DIR)/ovmf
OVMF_CODE    := $(OVMF_DIR)/code.fd
OVMF_VARS    := $(OVMF_DIR)/vars.fd
IMAGE_ROOT   := $(PROFILE_DIR)/image
ISO_ROOT     := $(PROFILE_DIR)/iso
SYMBOLS_DIR  := $(PROFILE_DIR)/symbols
IMAGE        := $(PROFILE_DIR)/$(IMAGE_NAME).img
ISO          := $(PROFILE_DIR)/$(IMAGE_NAME).iso

RUST_HOST    := $(shell rustc -vV | sed -n 's/host: //p')
LLVM_OBJCOPY := $(shell rustc --print sysroot)/lib/rustlib/$(RUST_HOST)/bin/llvm-objcopy

.PHONY: all build release clippy deps check-deps install-deps scripts kernel limine image iso symbols run debug clean distclean ovmf

all: image iso

build: kernel

clippy:
	RUSTFLAGS="-Zunstable-options -A unused_features" cargo clippy --workspace -Zjson-target-spec -- --no-deps

release:
	$(MAKE) BUILD_PROFILE=release all

scripts:
	chmod +x $(CHECK_DEPS) $(INSTALL_DEPS)

deps: check-deps

check-deps: scripts
	./$(CHECK_DEPS)

install-deps: scripts
	./$(INSTALL_DEPS)

kernel:
	cargo build --profile $(BUILD_PROFILE) -Zjson-target-spec

limine:
	if [ ! -d $(LIMINE_DIR) ]; then \
	    git clone https://github.com/limine-bootloader/limine.git \
	        --branch=v9.x-binary --depth=1 $(LIMINE_DIR); \
	fi
	$(MAKE) -C $(LIMINE_DIR)

ovmf:
	mkdir -p $(OVMF_DIR)
	cp /usr/share/AAVMF/AAVMF_CODE.fd $(OVMF_CODE) 2>/dev/null || \
	    cp /usr/share/qemu/edk2-aarch64-code.fd $(OVMF_CODE) 2>/dev/null || true
	cp /usr/share/AAVMF/AAVMF_VARS.fd $(OVMF_VARS) 2>/dev/null || \
	    cp /usr/share/qemu/edk2-arm-vars.fd $(OVMF_VARS) 2>/dev/null || true

symbols: kernel
	mkdir -p $(SYMBOLS_DIR)
	$(LLVM_OBJCOPY) --only-keep-debug $(KERNEL_BIN) $(SYMBOLS_DIR)/$(IMAGE_NAME).dbg
	cp $(KERNEL_BIN) $(SYMBOLS_DIR)/$(IMAGE_NAME).stripped
	$(LLVM_OBJCOPY) --strip-debug --add-gnu-debuglink=$(SYMBOLS_DIR)/$(IMAGE_NAME).dbg $(SYMBOLS_DIR)/$(IMAGE_NAME).stripped

image: kernel limine
	rm -rf $(IMAGE_ROOT)
	mkdir -p $(IMAGE_ROOT)/EFI/BOOT $(IMAGE_ROOT)/boot
	cp $(KERNEL_BIN) $(IMAGE_ROOT)/boot/kernel
	cp limine.conf $(IMAGE_ROOT)/boot/limine.conf
	cp $(LIMINE_DIR)/BOOTAA64.EFI $(IMAGE_ROOT)/EFI/BOOT/BOOTAA64.EFI
	mkdir -p $(PROFILE_DIR)
	rm -f $(IMAGE)
	dd if=/dev/zero of=$(IMAGE) bs=1M count=64
	sgdisk $(IMAGE) -n 1:2048:0 -t 1:ef00 -m 1
	mformat -i $(IMAGE)@@1M -F ::
	mmd -i $(IMAGE)@@1M ::/EFI ::/EFI/BOOT ::/boot
	mcopy -i $(IMAGE)@@1M $(IMAGE_ROOT)/EFI/BOOT/BOOTAA64.EFI ::/EFI/BOOT/
	mcopy -i $(IMAGE)@@1M $(IMAGE_ROOT)/boot/kernel ::/boot/
	mcopy -i $(IMAGE)@@1M $(IMAGE_ROOT)/boot/limine.conf ::/boot/

iso: kernel limine
	rm -rf $(ISO_ROOT)
	mkdir -p $(ISO_ROOT)/boot/limine $(ISO_ROOT)/EFI/BOOT
	cp $(KERNEL_BIN) $(ISO_ROOT)/boot/kernel
	cp limine.conf $(ISO_ROOT)/boot/limine.conf
	cp $(LIMINE_DIR)/limine-uefi-cd.bin $(ISO_ROOT)/boot/limine/
	cp $(LIMINE_DIR)/BOOTAA64.EFI $(ISO_ROOT)/EFI/BOOT/BOOTAA64.EFI
	mkdir -p $(PROFILE_DIR)
	xorriso -as mkisofs -R -r -J \
	    --efi-boot boot/limine/limine-uefi-cd.bin \
	    -efi-boot-part --efi-boot-image --protective-msdos-label \
	    $(ISO_ROOT) -o $(ISO)

run: image ovmf
	qemu-system-aarch64 \
		-M virt,acpi=off,gic-version=2 \
		-cpu cortex-a72 \
		-m 512M \
		-drive file=$(OVMF_CODE),if=pflash,format=raw,readonly=on \
		-drive file=$(OVMF_VARS),if=pflash,format=raw \
		-fw_cfg name=opt/org.tianocore/BootTimeout,string=0 \
		-drive file=$(IMAGE),format=raw,if=none,id=hd0 \
		-device virtio-blk-pci,drive=hd0 \
		-device ramfb \
		-device virtio-gpu-pci \
		-display gtk \
		-rtc base=utc \
		-serial stdio

debug: image ovmf
	qemu-system-aarch64 \
		-M virt,acpi=off,gic-version=2 \
		-cpu cortex-a72 \
		-m 512M \
		-drive file=$(OVMF_CODE),if=pflash,format=raw,readonly=on \
		-drive file=$(OVMF_VARS),if=pflash,format=raw \
		-fw_cfg name=opt/org.tianocore/BootTimeout,string=0 \
		-drive file=$(IMAGE),format=raw,if=none,id=hd0 \
		-device virtio-blk-pci,drive=hd0 \
		-device ramfb \
		-device virtio-gpu-pci \
		-display gtk \
		-rtc base=utc \
		-serial stdio \
		-S -gdb tcp::1234

clean:
	cargo clean
	rm -rf $(BUILD_DIR)
