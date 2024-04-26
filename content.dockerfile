
# Runtime
FROM gcr.io/distroless/cc-debian12

# asset_server
COPY content_server /usr/local/bin/server

# copy over assets
COPY target/cyberlith_content /usr/local/bin/files

CMD ["server"]