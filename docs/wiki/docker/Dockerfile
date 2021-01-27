FROM alpine:3.13 as runtime-env

EXPOSE 80

USER root
RUN apk update && apk upgrade
RUN apk add make git ca-certificates
RUN apk add --no-cache gcc musl-dev
RUN apk add --no-cache rust cargo

RUN cargo install mdbook --version=0.4.6

# RUN addgroup -g 1000 app && adduser -D -s /bin/sh -u 1000 -G app app
# USER app

WORKDIR /app/content
ENTRYPOINT ["/root/.cargo/bin/mdbook", "serve", "-d=/tmp/mdbook", "-p=80", "-n=0.0.0.0"]
