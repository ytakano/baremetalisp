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

I have developed some libraries for the OS.

### Trusted Firmware binary

- sunxi-a64-h5-spl.bin
  - https://github.com/apritzel/pine64/tree/master/binaries

and customized BL31 image, which is compiled from [ATF](https://github.com/ARM-software/arm-trusted-firmware)

```
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
