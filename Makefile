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

ASM_FILE=asm/aarch64.S
ASM_OBJ=aarch64.o

TARGET=aarch64-unknown-none

RUSTLIB=target/$(TARGET)/release/libbaremetalisp.a
RUSTFLAGS=$(RUSTC_MISC_ARGS)

CC=aarch64-linux-gnu-gcc
LD=aarch64-linux-gnu-ld

all: kernel8.img

$(ASM_OBJ): $(ASM_FILE)
	$(CC) -c $(ASM_FILE) -o $(ASM_OBJ) -D$(BSP)

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
