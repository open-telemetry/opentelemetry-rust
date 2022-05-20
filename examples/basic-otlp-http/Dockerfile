FROM rust:1.51
COPY . /usr/src/basic-otlp-http/
WORKDIR /usr/src/basic-otlp-http/
RUN cargo build --release
RUN cargo install --path .
CMD ["/usr/local/cargo/bin/basic-otlp-http"]
