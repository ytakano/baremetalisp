all: baremetalisp

AARCH64_ASM=asm/aarch64.as
X86_64_ASM=asm/x86_64.s

RUSTLIB=target/debug/libbaremetalisp.a

x86_64.o: $(X86_64_ASM)
	as $(X86_64_ASM) -o x86_64.o

$(RUSTLIB): FORCE
	cargo build

baremetalisp: $(RUSTLIB) x86_64.o
	cargo build
	ld -o baremetalisp x86_64.o $(RUSTLIB)

clean:
	cargo clean
	rm -f baremetalisp

FORCE: