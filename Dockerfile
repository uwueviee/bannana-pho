# This file is a template, and might need editing before it works on your project.
FROM rust:1.58 as builder

WORKDIR /usr/src/bannana-pho

COPY . .
RUN cargo build --release

FROM debian:buster-slim

COPY --from=builder /usr/src/bannana-pho/target/release/bannana-pho .

EXPOSE 3621
CMD ["./bannana-pho"]
