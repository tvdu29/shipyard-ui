FROM rust:buster as build

RUN cargo install trunk && \
    rustup target add wasm32-unknown-unknown && \
    rustup toolchain install nightly && \
    rustup default nightly && \
    rustup target add wasm32-unknown-unknown;


COPY backend /backend/
COPY frontend /frontend/
COPY src /src/
COPY Cargo.toml .

RUN rustup run nightly trunk build --release /frontend/index.html && \
    cargo +nightly install --path backend && \
    which backend;

FROM rust:buster as run

RUN DEBIAN_FRONTEND=noninteractive apt update -y && \
    DEBIAN_FRONTEND=noninteractive apt install python3-pip apt-utils nginx -y && \
    python3 -m pip install supervisor;

COPY --from=build /usr/local/cargo/bin/backend /usr/local/cargo/bin/backend
COPY --from=build /frontend/dist /dist

COPY nginx.conf /etc/nginx/sites-enabled/default

COPY supervisord.conf /etc/supervisor/supervisord.conf

CMD ["/usr/local/bin/supervisord"]