# cyberlith

Build Docker image for content_server:
```
cargo build --release --features local --manifest-path apps/content/Cargo.toml && cp target/release/content_server content_server && docker build --build-arg server_name=content_server --progress=plain -t content_image . && rm content_server
```