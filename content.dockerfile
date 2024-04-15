
# Runtime
FROM gcr.io/distroless/cc-debian12

# content_server
COPY content_server /usr/local/bin/server

# launcher
COPY launcher.html /usr/local/bin/files/launcher.html
COPY launcher.js /usr/local/bin/files/launcher.js
COPY launcher_bg.wasm /usr/local/bin/files/launcher_bg.wasm

# game
COPY game.html /usr/local/bin/files/game.html
COPY game.js /usr/local/bin/files/game.js
COPY game_bg.wasm /usr/local/bin/files/game_bg.wasm

CMD ["server"]