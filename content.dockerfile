
# Runtime
FROM gcr.io/distroless/cc-debian12

COPY content_server /usr/local/bin/server
COPY index.html /usr/local/bin/index.html
COPY game_client.js /usr/local/bin/game_client.js
COPY game_client_bg.wasm /usr/local/bin/game_client_bg.wasm

CMD ["server"]