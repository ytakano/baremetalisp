TARGET=raspi4

ASM_FILE=asm/aarch64.s
ASM_OBJ=aarch64.o

RUSTLIB=target/aarch64-unknown-linux-gnu/debug/libbaremetalisp.a
CLIB=csrc/libbaremetal_c.a

AS=aarch64-linux-gnu-as
LD=aarch64-linux-gnu-ld

all: kernel8.img

$(ASM_OBJ): $(ASM_FILE)
	$(AS) $(ASM_FILE) -o $(ASM_OBJ)

$(RUSTLIB): FORCE
	cargo build --features $(TARGET) --target aarch64-unknown-linux-gnu

$(CLIB): FORCE
	$(MAKE) -C csrc TARGET=$(TARGET)

baremetalisp: $(RUSTLIB) $(ASM_OBJ) $(CLIB)
	$(LD) -T link.ld -o baremetalisp $(ASM_OBJ) $(CLIB) $(RUSTLIB)

kernel8.img: baremetalisp
	aarch64-linux-gnu-objcopy -O binary baremetalisp kernel8.img

clean:
	cargo clean
	rm -f baremetalisp kernel8.img
	$(MAKE) -C csrc clean

FORCE: