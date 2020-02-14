all: baremetalisp

target/debug/libbaremetalisp.a:
	cargo build

baremetalisp: target/debug/libbaremetalisp.a
	ld -o baremetalisp target/debug/libbaremetalisp.a

clean:
	cargo clean
	rm -f baremetalisp