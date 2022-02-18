# This file is a template, and might need editing before it works on your project.
FROM rust:1.58 as builder

WORKDIR /usr/src/lvsp-server

COPY . .
RUN cargo build --release

FROM debian:buster-slim

COPY --from=builder /usr/src/lvsp-server/target/release/lvsp-server .

EXPOSE 3621
CMD ["./lvsp-server"]
