# Default to the RPi4
ifndef BSP
	BSP = raspi4
endif

# BSP-specific arguments
ifeq ($(BSP),raspi3)
	RUSTC_MISC_ARGS = -C target-cpu=cortex-a53
else ifeq ($(BSP),raspi4)
	RUSTC_MISC_ARGS = -C target-cpu=cortex-a72
endif

ASM_FILE=asm/aarch64.s
ASM_OBJ=aarch64.o

TARGET=aarch64-unknown-none-softfloat

RUSTLIB=target/$(TARGET)/release/libbaremetalisp.a
RUSTFLAGS=$(RUSTC_MISC_ARGS)

AS=aarch64-linux-gnu-as
LD=aarch64-linux-gnu-ld

all: kernel8.img

$(ASM_OBJ): $(ASM_FILE)
	$(AS) $(ASM_FILE) -o $(ASM_OBJ)

$(RUSTLIB): FORCE
	RUSTFLAGS="$(RUSTFLAGS)" cargo xrustc --features $(BSP) --target $(TARGET) --release

doc:
	cargo xdoc --target=$(TARGET) --features $(BSP) --document-private-items

baremetalisp: $(RUSTLIB) $(ASM_OBJ)
	$(LD) -T link.ld -o baremetalisp $(ASM_OBJ) $(RUSTLIB)

kernel8.img: baremetalisp
	aarch64-linux-gnu-objcopy -O binary baremetalisp kernel8.img

clean:
	cargo clean
	rm -f baremetalisp kernel8.img

FORCE: