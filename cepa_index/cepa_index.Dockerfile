FROM rust

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

RUN mkdir -p /opt/cepa_index

COPY . /opt/cepa_index

RUN cargo build --release  --manifest-path /opt/cepa_index/Cargo.toml

EXPOSE 8000

ENTRYPOINT ["cargo", "run","--release",  "--manifest-path", "/opt/cepa_index/Cargo.toml"]