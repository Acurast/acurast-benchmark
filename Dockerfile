FROM rust:1.84.0 AS prepare

WORKDIR /usr/src/acubench

RUN USER=root cargo init --lib
COPY ./rust/Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs
RUN rm ./target/release/deps/acubench*

FROM rust:1.84.0 AS build

WORKDIR /usr/src/acubench

COPY ./rust .
COPY --from=prepare /usr/src/acubench/target ./target
RUN cargo build --release

CMD ["cargo", "test"]