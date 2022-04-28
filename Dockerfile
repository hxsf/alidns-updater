# syntax=docker/dockerfile:experimental
FROM rust:1.60-alpine as builder

COPY . /code
WORKDIR /code

RUN  --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/code/target \
    sed -i 's/dl-cdn\.alpinelinux\.org/mirrors.aliyun.com/g' /etc/apk/repositories \
    && apk add --no-cache musl-dev ca-certificates \
    && cargo build --release --target x86_64-unknown-linux-musl \
    && cp /code/target/x86_64-unknown-linux-musl/release/alidns-updater /alidns-updater

FROM alpine:3

RUN sed -i 's/dl-cdn\.alpinelinux\.org/mirrors.aliyun.com/g' /etc/apk/repositories \
    && apk add --no-cache bash ca-certificates tzdata \
    && cp /usr/share/zoneinfo/Asia/Shanghai /etc/localtime \
    && echo 'Asia/Shanghai' > /etc/timezone
COPY --from=builder /alidns-updater /alidns-updater

CMD ["/alidns-updater"]




# FROM rust:1.60-bullseye
# ENV RUSTUP_DIST_SERVER="https://rsproxy.cn" RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"
# COPY . /code
# WORKDIR /code
# RUN sed -i 's@deb http://(deb|security).debian.org@deb http://mirrors.hxsf.work@g' /etc/apt/sources.list \
#     && apt update \
#     && apt install -y musl-tools build-essential ca-certificates \
#     && apt clean && rm -rf /var/lib/apt/lists/* \
#     && rustup target add x86_64-unknown-linux-musl \
#     && cargo build --release --target x86_64-unknown-linux-musl

# CMD ["/code/target/x86_64-unknown-linux-musl/release/alidns-updater"]
