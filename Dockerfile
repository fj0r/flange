ARG BASEIMAGE=ghcr.io/fj0r/faucet_dx:latest
FROM ${BASEIMAGE} as build
FROM scratch
COPY --from=build /app/target/release/flange /app
COPY config.toml /app
