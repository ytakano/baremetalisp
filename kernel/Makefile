# Default to the Pine64
ifndef $(BSP)
	BSP = pine64
endif

# 2MiB Stack
STACKSIZE = 1024 * 1024 * 2

# 4 CPUs
NUMCPU = 4

# BSP-specific arguments
ifeq ($(BSP),raspi3)
	RUSTC_MISC_ARGS = -C target-cpu=cortex-a53
	INITADDR = 0x80000
else ifeq ($(BSP),raspi4)
	RUSTC_MISC_ARGS = -C target-cpu=cortex-a72
	INITADDR = 0
else ifeq ($(BSP),pine64)
	RUSTC_MISC_ARGS = -C target-cpu=cortex-a53
	INITADDR = 0x40000000
endif

ASM_FILE_DEP=asm/device/raspi.S asm/device/pine64.S asm/cache_helper.S

ASM_FILE=asm/boot.S
ASM_OBJ=boot.o

TARGET=aarch64-unknown-none-softfloat

RUSTLIB=target/$(TARGET)/release/libbaremetalisp.a
RUSTFLAGS=$(RUSTC_MISC_ARGS)

ifndef $(CC)
	CC = clang
endif

ifndef $(LD)
	LD = rust-lld -flavor gnu
endif

all: kernel8.img

$(ASM_OBJ): $(ASM_FILE) $(ASM_FILE_DEP)
	$(CC) --target=aarch64-elf -c $(ASM_FILE) -o $(ASM_OBJ) -D$(BSP) -DSTACKSIZE="$(STACKSIZE)"

$(RUSTLIB): FORCE
	RUSTFLAGS="$(RUSTFLAGS)" cargo xrustc --features $(BSP) --target $(TARGET) --release

doc:
	cargo xdoc --target=$(TARGET) --features $(BSP) --document-private-items

link.ld.$(BSP): link.ld
	sed "s/#INITADDR#/$(INITADDR)/" link.ld | sed "s/#STACKSIZE#/$(STACKSIZE)/" | sed "s/#NUMCPU#/$(NUMCPU)/" > link.ld.$(BSP)

baremetalisp: $(RUSTLIB) $(MMU_OBJ) $(ASM_OBJ) link.ld.$(BSP)
	$(LD) --gc-sections -m aarch64elf -nostdlib -T link.ld.$(BSP) -o baremetalisp $(ASM_OBJ) $(RUSTLIB)

kernel8.img: baremetalisp
	rust-objcopy -O binary baremetalisp kernel8.img

# for USB boot of Pine64
SUNXI_FEL=sunxi-fel
boot64:
	sudo ${SUNXI_FEL} spl sunxi-a64-h5-spl.bin
	sudo ${SUNXI_FEL} write 0x44000 bl31.bin
	sudo ${SUNXI_FEL} write ${INITADDR} kernel8.img
	sudo ${SUNXI_FEL} reset64 0x44000

rmobj: FORCE
	rm -f baremetalisp kernel8.img *.o

clean:
	cargo clean
	rm -f baremetalisp kernel8.img *.o link.ld.*

FORCE:
