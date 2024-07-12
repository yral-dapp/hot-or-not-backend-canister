FROM ghcr.io/dfinity/icp-dev-env:latest as setup

WORKDIR /app

COPY . .

RUN apt-get install -y bsdmainutils
RUN ./scripts/canisters/docker/install_all_canisters.sh

FROM ghcr.io/dfinity/icp-dev-env:latest as runner

WORKDIR /app
RUN mkdir -p /root/.local/share/dfx/network/local
COPY --from=setup /root/.local/share/dfx/network/local/state /root/.local/share/dfx/network/local/

CMD ["dfx", "start", "--host", "0.0.0.0:4943", "--artificial-delay", "0"]
