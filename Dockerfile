FROM clux/muslrust:nightly as builder
ARG BINARY
LABEL Author=TommyLike<tommylikehu@gmail.com>
WORKDIR /app
COPY . /app


RUN cargo +nightly build --release --bin $BINARY --target x86_64-unknown-linux-musl
RUN cargo install sqlx-cli --no-default-features --features native-tls,mysql,openssl-vendored --root /app

FROM openeuler/openeuler:22.03
ARG BINARY
ARG CONFIG=./config
ENV BINARY=${BINARY}
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/$BINARY /app
COPY --from=builder /app/bin/sqlx /app
COPY $CONFIG /app/config
COPY ./migrations /app/migrations
ENTRYPOINT /app/$(echo $BINARY)
