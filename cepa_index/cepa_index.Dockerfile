FROM rust

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

RUN mkdir -p /opt/cepa/cepa_index
RUN mkdir -p /opt/cepa/cepa_common

COPY ./cepa_index /opt/cepa/cepa_index
COPY ./cepa_common /opt/cepa/cepa_common

RUN cargo build --release  --manifest-path /opt/cepa/cepa_index/Cargo.toml

EXPOSE 8000

ENTRYPOINT ["cargo", "run","--release",  "--manifest-path", "/opt/cepa/cepa_index/Cargo.toml"]