DOCKER_IMG := datahearth/clear-docker-images
GHCR_IMG := ghcr.io/${DOCKER_IMG}
VERSION := 0.4.1

.PHONY: initialize-linux-build
initialize-linux-build:
	@brew install FiloSottile/musl-cross/musl-cross
	@rustup target add x86_64-unknown-linux-musl
	@echo 'add these lines to your "~/.cargo":'
	@printf '\n[target.x86_64-unknown-linux-musl]\nlinker = "x86_64-linux-musl-gcc"\n'

.PHONY: build-docker
build-docker:
	@docker build --tag ${DOCKER_IMG}:${VERSION} .
	@docker tag ${DOCKER_IMG}:${VERSION} ${GHCR_IMG}:${VERSION}
	@docker tag ${DOCKER_IMG}:${VERSION} ${DOCKER_IMG}:latest
	@docker tag ${DOCKER_IMG}:${VERSION} ${GHCR_IMG}:latest

.PHONY: build-binaries
build-binaries:
	@echo "Building Linux MUSL binary..."
	@cargo build --release --target x86_64-unknown-linux-musl
	@echo "Building MacOS darwin" 
	@cargo build --release --target x86_64-apple-darwin

.PHONY: push-docker-images
push-docker-images: build-docker
	@docker push --all-tags ${GHCR_IMG}
	@docker push --all-tags ${DOCKER_IMG}

.PHONY: bump-version
bump-version:
	@echo "Bump version \"${VERSION}\" to \"${NEW_VERSION}\""
	@sd "version = \"${VERSION}\"" "version = \"${NEW_VERSION}\"" Cargo.toml
	@sd "VERSION := ${VERSION}" "VERSION := ${NEW_VERSION}" Makefile
	@git add .
	@git commit -m "bump v${NEW_VERSION}"
	@git tag -m "bump v${NEW_VERSION}" v${NEW_VERSION}
	@git push --follow-tags
