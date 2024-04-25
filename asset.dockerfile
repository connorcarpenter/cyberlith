

#FROM ubuntu:latest

# Install bash and libgcc1
#RUN apt-get update && \
#    apt-get install -y bash libgcc1 && \
#    rm -rf /var/lib/apt/lists/*

# Runtime
FROM gcr.io/distroless/cc-debian12

# asset_server
COPY asset_server /usr/local/bin/server

# copy over assets
COPY target/cyberlith_assets /usr/local/bin/assets

CMD ["server"]