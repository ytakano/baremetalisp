FROM alpine:3.11.3

RUN apk update && \
    apk upgrade && \
    apk add gcc zsh zsh-vcs git curl

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly -y

COPY zshenv /root/.zshenv
COPY zshrc /root/.zshrc

ENTRYPOINT ["zsh"]
