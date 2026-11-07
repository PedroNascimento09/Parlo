FROM debian:bookworm-slim

ARG PARLO_VERSION=0.2.6
ARG PARLO_SHA256=c0b1556ab2ce38be58cdc2ee063c2cf7c9ac7e517a7c78855a5eb9259b356d07

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/parlo

RUN curl -fsSL -o parlo-linux-x86_64.tar.gz "https://github.com/PedroNascimento09/Parlo/releases/download/v${PARLO_VERSION}/parlo-linux-x86_64.tar.gz" \
 && echo "${PARLO_SHA256}  parlo-linux-x86_64.tar.gz" | sha256sum -c - \
 && tar -xzf parlo-linux-x86_64.tar.gz \
 && rm parlo-linux-x86_64.tar.gz \
 && chmod 755 parlo parlo-app

RUN groupadd --gid 10001 parlo \
 && useradd --uid 10001 --gid 10001 --home-dir /data --shell /usr/sbin/nologin parlo \
 && mkdir -p /data \
 && chown 10001:10001 /data /opt/parlo

COPY --chown=10001:10001 packaging/docker/entrypoint.sh /usr/local/bin/parlo-entrypoint
RUN chmod 755 /usr/local/bin/parlo-entrypoint

USER 10001:10001

ENV PARLO_PORT=8787 \
    PARLO_CONFIG=/data/config.json \
    PARLO_DATA_DIR=/data

EXPOSE 8787
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/parlo-entrypoint"]
