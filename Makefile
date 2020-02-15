all: baremetalisp

ASM_FILE=asm/aarch64.s
#ASM_FILE=asm/x86_64.s
ASM_OBJ=aarch64.o

RUSTLIB=target/aarch64-unknown-linux-gnu/debug/libbaremetalisp.a

AS=aarch64-linux-gnu-as
LD=aarch64-linux-gnu-ld

$(ASM_OBJ): $(ASM_FILE)
	$(AS) $(ASM_FILE) -o $(ASM_OBJ)

$(RUSTLIB): FORCE
	cargo build --target aarch64-unknown-linux-gnu

baremetalisp: $(RUSTLIB) $(ASM_OBJ)
	$(LD) -o baremetalisp $(ASM_OBJ) $(RUSTLIB)

clean:
	cargo clean
	rm -f baremetalisp

FORCE: