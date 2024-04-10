####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=sveio
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /sveio
COPY ./ .
RUN cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /sveio

# Copy our build
COPY --from=builder /sveio/target/x86_64-unknown-linux-musl/release/sveio ./
COPY --from=builder /sveio/capitals.csv ./
COPY --from=builder /sveio/frontend ./frontend
# Use an unprivileged user.
USER sveio:sveio

CMD ["/sveio/sveio"]