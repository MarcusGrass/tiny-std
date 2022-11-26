FROM arm64v8/debian:testing-slim
COPY ./qemu-aarch64-static /usr/bin/qemu-aarch64

WORKDIR /tiny-std

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get -y update && \
    apt-get -y install \
    pkg-config \
    build-essential \
    curl

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup toolchain install nightly
RUN cargo install lazy_static; exit 0

# CMD [".local/all_tests.sh"]
ENTRYPOINT [".local/aarch64-test-runners.sh"]
