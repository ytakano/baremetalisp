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

In Docker CONTAINER!
```
$ cd /hostdir/kernel
$ make
```

build for Raspberry Pi 3
```
$ make clean
$ make BSP=raspi3
```

## Run in Qemu on Host

On HOST! Use Raspberry Pi 3's kernel image.
```
$ qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial stdio -d int
```

## Status of Nightly Rust

https://rust-lang.github.io/rustup-components-history/aarch64-unknown-linux-gnu.html
