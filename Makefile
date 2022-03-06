VERSION := 0.4.2

.PHONY: bump-version
bump-version:
	@type sd > /dev/null
	@echo "Bump version to \"${NEW_VERSION}\""
	@sd "version = \"${VERSION}\"" "version = \"${NEW_VERSION}\"" Cargo.toml
	@sd "VERSION=${VERSION}" "VERSION=${NEW_VERSION}" .env
	@sd "VERSION := ${VERSION}" "VERSION := ${NEW_VERSION}" Makefile
	@cargo update --package=clear-docker-images
	@git add .
	@git commit -m "bump v${NEW_VERSION}"
	@git tag -m "bump v${NEW_VERSION}" v${NEW_VERSION}
	@git push --follow-tags
