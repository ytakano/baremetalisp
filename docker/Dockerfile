FROM ubuntu:22.04

RUN apt-get -y update && \
    apt-get -y upgrade && \
    apt-get -y install build-essential zsh git curl libncurses5-dev libtinfo5 clang swig python3-pip

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly -y
RUN /bin/bash -c "source $HOME/.cargo/env" && \
    export PATH=$HOME/.cargo/bin:$PATH && \
    rustup component add rust-src llvm-tools-preview && \
    cargo install cargo-xbuild cargo-binutils

COPY zshenv /root/.zshenv
COPY zshrc /root/.zshrc

ENTRYPOINT ["zsh"]
