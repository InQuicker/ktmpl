FROM scratch

ENTRYPOINT ["/ktmpl"]

COPY target/x86_64-unknown-linux-musl/release/ktmpl /ktmpl
