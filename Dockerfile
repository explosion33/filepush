FROM rust:1.65.0


COPY web .

RUN cargo build --release

EXPOSE 80

CMD ["./target/release/filepush"]