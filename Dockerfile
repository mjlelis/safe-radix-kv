# Dockerfile para safe-radix-kv
# Ambiente completo com Clang, Bear, C2Rust, Rust Nightly, Miri e Cargo-Fuzz

FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    clang \
    llvm \
    libclang-dev \
    bear \
    curl \
    git \
    pkg-config \
    valgrind \
    python3 \
    python3-pip \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Instalação Rust nightly e ferramentas essenciais para verificação formal
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly \
    && rustup component add miri rust-src clippy rustfmt \
    && cargo install cargo-fuzz || true

WORKDIR /workspace
COPY . /workspace

CMD ["cargo", "+nightly", "miri", "test"]
