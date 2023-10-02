WORKDIR /usr/src/ca_server

RUN rustup default nightly-2023-03-22

COPY . .
COPY .cargo .cargo
COPY ./docker/init.sh ./init.sh

RUN --mount=type=ssh  \
    --mount=type=cache,target=/root/.cargo/git  \
    --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,sharing=locked,target=/usr/src/ca_server/target \
    cargo install --bin ca_server --path ./ca_server

EXPOSE 8080

HEALTHCHECK CMD curl --fail http://localhost:8080/health || exit 1

CMD ["bash", "init.sh"]
