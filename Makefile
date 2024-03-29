# .env file has the variables $DOCKER_USERNAME and $PASSWORD defined
include .env

setup:
	cargo install cross
	rustup target add x86_64-unknown-linux-musl
	docker buildx create --use --platform=linux/arm64,linux/amd64 --name multi-platform-builder
	docker buildx inspect --bootstrap

build-dev:
	cargo build --release --locked --manifest-path=actix/Cargo.toml --target x86_64-unknown-linux-musl

docker-local: build-dev
	docker build --tag chhoto-url --build-arg TARGETARCH=amd64 -f Dockerfile.multiarch .

docker-test: docker-local
	docker ps -q --filter "name=chhoto-url" | xargs -r docker stop
	docker ps -aq --filter "name=chhoto-url" | xargs -r docker rm
	docker run -p 4567:4567 --name chhoto-url -e password="${PASSWORD}" -d chhoto-url
	docker logs chhoto-url -f

docker-dev: build-dev
	docker build --push --tag ${DOCKER_USERNAME}/chhoto-url:dev --build-arg TARGETARCH=amd64 -f Dockerfile.multiarch .

build-release:
	cross build --release --locked --manifest-path=actix/Cargo.toml --target aarch64-unknown-linux-musl
	cross build --release --locked --manifest-path=actix/Cargo.toml --target armv7-unknown-linux-musleabihf
	cross build --release --locked --manifest-path=actix/Cargo.toml --target x86_64-unknown-linux-musl

V_PATCH := $(shell cat actix/Cargo.toml | sed -rn 's/^version = "(.+)"$$/\1/p')
V_MINOR := $(shell cat actix/Cargo.toml | sed -rn 's/^version = "(.+)\..+"$$/\1/p')
V_MAJOR := $(shell cat actix/Cargo.toml | sed -rn 's/^version = "(.+)\..+\..+"$$/\1/p')
docker-release: build-release
	docker buildx build --push --tag ${DOCKER_USERNAME}/chhoto-url:${V_MAJOR} --tag ${DOCKER_USERNAME}/chhoto-url:${V_MINOR} \
		--tag ${DOCKER_USERNAME}/chhoto-url:${V_PATCH} --tag ${DOCKER_USERNAME}/chhoto-url:latest \
		--platform linux/amd64,linux/arm64,linux/arm/v7 -f Dockerfile.multiarch .

clean:
	docker ps -q --filter "name=chhoto-url" | xargs -r docker stop
	docker ps -aq --filter "name=chhoto-url" | xargs -r docker rm
	cargo clean --manifest-path=actix/Cargo.toml

.PHONY: build-dev docker-local build-release