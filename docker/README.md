## build and run Docker container

build & run
```
$ docker-compose build
$ docker-compose run baremetalisp
```

remove
```
$ docker-compose rm
```

## Compile Kernel in the container

Docker CONTAINER!
```
$ cd /hostdir/kernel
$ make
```

build for Raspberry Pi 3
```
$ vi link.ld
SECTIONS
{
    . = 0x80000;
/*    . = 0;*/

$ make clean
$ make BSP=raspi3
```

## Run in Qemu on Host

HOST! Use Raspberry Pi 3's kernel image.
```
$ qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial stdio -d
```

## Status of Nightly Rust

https://rust-lang.github.io/rustup-components-history/aarch64-unknown-linux-gnu.html
