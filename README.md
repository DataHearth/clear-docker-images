# clear-docker-images

`clear-docker-images` is a small rust binary made to cleanup old docker images but date, repository and tags.

## Usage

### Docker container

```bash
docker run --name clear-docker-image -v /var/run/docker.sock:/var/run/docker.sock ghcr.io/datahearth/clear-docker-image <OPTIONS>
```

### Binary

```bash
mkdir -p $HOME/.local/bin
export PATH=$PATH:$HOME/.local/bin

# Available distribution: linux | darwin
wget -o $HOME/.local/bin/clear-docker-images ~/. https://github.com/DataHearth/clear-docker-images/releases/download/v0.2.0/x86_x64-<DISTRIBUTION>-clear-docker-images

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

`--dry-run` : image cleanup will not be triggered  
`--force` : should docker force image removal (it may create orphan images)  
`-h, --help` : Print help information  
`-r, --repository <REPOSITORY>` : filter by repository name  
`-t, --tags <TAGS>` : add tags exclusion  
`-V, --version` : Print version information  
