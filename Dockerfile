FROM rust:alpine AS back-stage

RUN apk update
RUN apk add cmake make musl-dev g++

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Build image from scratch
FROM scratch
LABEL org.opencontainers.image.source = "https://github.com/CCC-MF/kafka-to-bwhc"
LABEL org.opencontainers.image.licenses = "AGPL-3.0-or-later"
LABEL org.opencontainers.image.description = "Sends request to bwHC-Backend, awaits response and sends it back"

COPY --from=back-stage /build/target/release/kafka-to-bwhc .
USER 65532:65532
CMD ["./kafka-to-bwhc"]