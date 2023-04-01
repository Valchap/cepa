FROM rust

RUN mkdir -p /opt/cepa_routing

COPY . /opt/cepa_routing

RUN cargo build --release  --manifest-path /opt/cepa_routing/Cargo.toml

ENTRYPOINT ["cargo", "run","--release",  "--manifest-path", "/opt/cepa_routing/Cargo.toml"]