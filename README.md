# baremetalisp

![CI](https://github.com/ytakano/baremetalisp/workflows/CI/badge.svg)

![demo](https://raw.githubusercontent.com/ytakano/baremetalisp/master/misc/gif/baremetalisp_demo.gif)


## Serial Console

- baud rate: 115200
- no parity
- 1 stop bit

```text
$ screen /dev/tty.usbserial-0001 115200
ctrl-a d (detach)
$ screen -r (attach)
ctrl-a k (kill)
```

## Dependencies

### Trusted Firmware binary

- sunxi-a64-h5-spl.bin
  - https://github.com/apritzel/pine64/tree/master/binaries

and customized BL31 image, which is compiled from [ATF](https://github.com/ARM-software/arm-trusted-firmware)

```text
$ make PLAT=sun50i_a64 SPD=opteed bl31
```

### BLisp

A statically typed programming language.

- https://ytakano.github.io/blisp/
- https://crates.io/crates/blisp
- https://github.com/ytakano/blisp

### synctools

A library for synchronization.

- https://crates.io/crates/synctools
- https://github.com/ytakano/synctools/

### memalloc

A memory allocator crate, which uses buddy and slab allocator.

- https://github.com/ytakano/memalloc

## Boot Image

Under Construction

### SCP

- or1k tool chain (for x86-64 only)
  - https://musl.cc/or1k-linux-musl-cross.tgz
- https://github.com/crust-firmware/crust

```text
$ export PATH="$PATH:/path/to/or1k-linux-musl-cross/bin/
$ git clone https://github.com/crust-firmware/crust
$ cd crust
$ export CROSS_COMPILE=or1k-linux-musl-
$ make pinephone_defconfig
$ make scp
$ ls build/scp/scp.bin
scp.bin
```

### u-boot

For x86-64.

```tex
$ export CROSS_COMPILE=aarch64-linux-gnu-
$ export ARCH=arm64
```

```text
$ git clone https://gitlab.com/pine64-org/u-boot.git
$ cd u-boot
$ git fetch origin v2021.04
$ git checkout v2021.04
$ cp /path/to/bl31.bin .
$ cp /path/to/scp.bin .
$ make distclean
$ make pinephone_defconfig
$ make all
```

## M1 Mac Qemu

Install pathced Qemu. https://github.com/knazarov/homebrew-qemu-virgl

```
$ brew install knazarov/qemu-virgl/qemu-virgl
```

Compile and run.

```text
$ cd kernel
$ make BSP=raspi3
$ qemu-system-aarch64 -M raspi3 -accel tcg,split-wx=on -kernel kernel8.img -serial stdio -d int
```
