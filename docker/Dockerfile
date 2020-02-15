FROM ubuntu:19.10

RUN apt-get -y update && \
    apt-get -y upgrade && \
    apt-get -y install build-essential zsh git curl libncurses5-dev libtinfo5 gcc-aarch64-linux-gnu

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly -y
RUN /bin/bash -c "source $HOME/.cargo/env" && export PATH=$HOME/.cargo/bin:$PATH && rustup target add aarch64-unknown-linux-gnu

COPY zshenv /root/.zshenv
COPY zshrc /root/.zshrc

ENTRYPOINT ["zsh"]
