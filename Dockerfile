FROM ghcr.io/dfinity/icp-dev-env:latest

WORKDIR /app

COPY . .

RUN apt-get install -y bsdmainutils
RUN ./scripts/canisters/docker/install_all_canisters.sh

CMD ["dfx", "start", "--host", "0.0.0.0:4943", "--artificial-delay", "0"]
