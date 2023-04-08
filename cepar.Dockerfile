FROM rust

RUN mkdir -p /opt/cepa/cepa_routing
RUN mkdir -p /opt/cepa/cepa_common

COPY ./cepa_routing /opt/cepa/cepa_routing
COPY ./cepa_common /opt/cepa/cepa_common

RUN cargo build --release  --manifest-path /opt/cepa/cepa_routing/Cargo.toml

ENTRYPOINT ["cargo", "run","--release",  "--manifest-path", "/opt/cepa/cepa_routing/Cargo.toml"]