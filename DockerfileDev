FROM rust as vpn-build

#WORKDIR /build
#COPY Cargo.toml .
#COPY dummy.rs ./src/main.rs
#RUN cargo build
#COPY src ./src
#touch src files to force compilation
#RUN find ./src -type f -exec touch {} +
#RUN cargo build

FROM python:3.11
RUN apt-get update
# install common helpful tools
RUN apt-get install -y curl vim jq net-tools htop iptables build-essential iputils-ping iproute2 dnsutils ncat
# install python requirements for yagna_mon.py
RUN pip install quart requests websockets scapy aiohttp
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN exec bash
WORKDIR /build
COPY Cargo.toml .
COPY dummy.rs ./src/main.rs
ENV CARGO_TARGET_DIR /target
RUN /root/.cargo/bin/cargo build
COPY src ./src
#touch src files to force compilation
RUN find ./src -type f -exec touch {} +
RUN /root/.cargo/bin/cargo build

WORKDIR /vpn
COPY *.sh .
#COPY --from=vpn-build /build/target/debug/novpn /usr/bin
