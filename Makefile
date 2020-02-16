all: kernel.img

TARGET=rspi3

ASM_FILE=asm/aarch64.s
ASM_OBJ=aarch64.o

RUSTLIB=target/aarch64-unknown-linux-gnu/debug/libbaremetalisp.a

AS=aarch64-linux-gnu-as
LD=aarch64-linux-gnu-ld

$(ASM_OBJ): $(ASM_FILE)
	$(AS) $(ASM_FILE) -o $(ASM_OBJ)

$(RUSTLIB): FORCE
	cargo build --features $(TARGET) --target aarch64-unknown-linux-gnu

baremetalisp: $(RUSTLIB) $(ASM_OBJ)
#	$(LD) -o baremetalisp $(ASM_OBJ) $(RUSTLIB)
	$(LD) -T link.ld -o baremetalisp $(ASM_OBJ) $(RUSTLIB)

kernel.img: baremetalisp
	aarch64-linux-gnu-objcopy -O binary baremetalisp kernel.img

clean:
	cargo clean
	rm -f baremetalisp kernel.img

FORCE: