.PHONY: bump-version
bump-version:
	@type sd > /dev/null
	@echo "Bump version to \"${NEW_VERSION}\""
	@sd "version = \"${VERSION}\"" "version = \"${NEW_VERSION}\"" Cargo.toml
	@sd "VERSION=${VERSION}" "VERSION=${NEW_VERSION}" .env
	@git add .
	@git commit -m "bump v${NEW_VERSION}"
	@git tag -m "bump v${NEW_VERSION}" v${NEW_VERSION}
	@git push --follow-tags
