FROM fedora:latest

RUN dnf install -y protobuf-compiler gcc

# ENV RUSTUP_HOME=/rust/rustup CARGO_HOME=/rust/cargo
# ENV PATH $CARGO_HOME/bin:$PATH
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --no-modify-path

COPY entrypoint.sh / 

WORKDIR /build
CMD ["/entrypoint.sh"]
