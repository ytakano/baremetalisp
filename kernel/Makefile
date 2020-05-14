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

CC=clang --target=aarch64-elf
LD=ld.lld -m aarch64elf -nostdlib 

all: kernel8.img

$(ASM_OBJ): $(ASM_FILE)
	$(CC) -c $(ASM_FILE) -o $(ASM_OBJ) -D$(BSP)

$(RUSTLIB): FORCE
	RUSTFLAGS="$(RUSTFLAGS)" cargo xrustc --features $(BSP) --target $(TARGET) --release

doc:
	cargo xdoc --target=$(TARGET) --features $(BSP) --document-private-items

baremetalisp: $(RUSTLIB) $(MMU_OBJ) $(ASM_OBJ)
	$(LD) -T link.ld -o baremetalisp $(ASM_OBJ) $(RUSTLIB)

kernel8.img: baremetalisp
	llvm-objcopy-10 -O binary baremetalisp kernel8.img

rmobj: FORCE
	rm -f baremetalisp kernel8.img *.o

clean:
	cargo clean
	rm -f baremetalisp kernel8.img *.o

FORCE:
