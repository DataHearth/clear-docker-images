# clear-docker-images

`clear-docker-images` is a small rust binary made to cleanup old docker images by date, repository and tags.

## Usage

By default, `clear-docker-images` will select images that are older than 2 days old. You can choose to customize its behavior by filtering a different date by passing 1 or 2 dates, a repository and tags.

### Docker container

```bash
docker run --name clear-docker-image -v /var/run/docker.sock:/var/run/docker.sock ghcr.io/datahearth/clear-docker-images <OPTIONS>
```

### Binary

```bash
mkdir -p $HOME/.local/bin
export PATH=$PATH:$HOME/.local/bin

# Available binaries:
# x86_64-unknown-linux-gnu | x86_64-unknown-linux-musl | x86_64-apple-darwin
wget -o $HOME/.local/bin/clear-docker-images ~/. https://github.com/DataHearth/clear-docker-images/releases/download/<VERSION>/<BINARY>

clear-docker-images <OPTIONS>
```

### Source

```bash
git clone https://github.com/DataHearth/clear-docker-images.git
cd clear-docker-images
cargo install --path .

clear-docker-images <OPTIONS>
```

## Options

```bash
USAGE:
    clear-docker-images [OPTIONS] [REPOSITORY]

ARGS:
    <REPOSITORY>
            filter by repository name

OPTIONS:
    -d, --date <DATE>
            filter by date.

            Can filter by a minimum age $DATE or from $START|$STOP (format example: YYYY-MM-DD or
            YYYY-MM-DDTHH:MM:SS) [default: $NOW - 2d]

        --dry-run
            image cleanup will not be triggered [default: false]

        --force
            force image removal [default: false]

    -h, --help
            Print help information

    -t, --tags <TAGS>
            add tags exclusion

    -v, --verbose
            add more logs [default: false]

    -V, --version
            Print version information
```
