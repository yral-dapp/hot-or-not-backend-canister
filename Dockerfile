FROM ghcr.io/dfinity/icp-dev-env:latest

WORKDIR /app

COPY . .

RUN dfx identity import --storage-mode=plaintext admin ./scripts/canisters/docker/local-admin.pem
RUN dfx identity use admin

RUN apt-get update && apt-get install -y bsdmainutils parallel --no-install-recommends && \
  rm -rf /var/lib/apt/lists/*

RUN ./scripts/canisters/docker/deploy.sh

CMD ["dfx", "start", "--host", "0.0.0.0:4943", "--artificial-delay", "0"]
