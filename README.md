# clear-docker-images

`clear-docker-images` is a small rust binary made to cleanup old docker images but date, repository and tags.

## Usage

By default (it will change in the futur), `clear-docker-images` will select images that are older than 2 days old. You can choose to customize which repository will be selected and if specific tags should be ignored.

### Docker container

```bash
docker run --name clear-docker-image -v /var/run/docker.sock:/var/run/docker.sock ghcr.io/datahearth/clear-docker-images <OPTIONS>
```

### Binary

```bash
mkdir -p $HOME/.local/bin
export PATH=$PATH:$HOME/.local/bin

# Available distribution: linux | darwin
wget -o $HOME/.local/bin/clear-docker-images ~/. https://github.com/DataHearth/clear-docker-images/releases/download/<VERSION>/x86_x64-<DISTRIBUTION>-clear-docker-images

clear-docker-images <OPTIONS>
```

### Source

```bash
git clone https://github.com/DataHearth/clear-docker-images.git
cd clear-docker-images
cargo install --path .

clear-docker-images <OPTIONS>
```

## Flags

```bash
  -d, --date <DATE>                filter by repository name (ISO 8601) [default: $NOW - 2 days]
      --dry-run                    image cleanup will not be triggered
      --force                      should docker force image removal (it may create orphan images)
  -h, --help                       Print help information
  -r, --repository <REPOSITORY>    filter by repository name
  -t, --tags <TAGS>                add tags exclusion Example: -t 1.1.0 -t release
  -v, --verbose                    add more logs
  -V, --version                    Print version information
```
