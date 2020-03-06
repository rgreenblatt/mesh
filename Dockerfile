FROM rust

WORKDIR /root/
RUN git clone https://github.com/rgreenblatt/mesh
WORKDIR /root/mesh/
RUN cargo build --release
RUN mkdir bin
RUN cp target/release/mesh bin

