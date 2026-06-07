FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive
ENV HOME=/root
RUN apt-get update && apt-get install -y \
    curl ca-certificates build-essential pkg-config \
    libssl-dev libpq-dev \
    && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:$PATH"
WORKDIR /src
COPY . /src
RUN CXXFLAGS="" cargo build --release -p fav 2>&1
