<p align="center">
  <a href="https://consciousnessarchive.com">
    <img alt="Consciousness Archive" src="./logo.png" />
  </a>
</p>

[//]: # (# Consciousness Archive)


# Consciousness Archive Server
Server to deliver consciousness calibrations, images, videos, audio, article/course markdown files, and more.
Files are stored in local memory.


Initialize Database
```shell
cargo make upsert_articles
```

## Run Server
```shell
cargo run -r -p ca_server
```

## Run Admin
```shell
cargo run -r -p ca_admin -t <file_type> -f <path> -n <name> -i <image_url>
```

#### Convert Evernote Article to Markdown
[evernote2md](https://github.com/wormi4ok/evernote2md)
```shell
brew install evernote2md

scripts/evernote2md.sh --input some_evernote.enex
```


## Remote Deploy to VM
```bash
# GitHub, manage terminal processes, and Cargo build dependencies
sudo apt install -y git screen build-essential libssl-dev libsasl2-dev pkg-config libfontconfig libfontconfig1-dev

# Install Rust
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"

# Set GitHub remote
git clone https://github.com/0xBreath/ca_backend.git
git reset --hard origin/main
git pull origin main

# Create a screen to run the server
screen -R server
# Start the server
cargo run -r -p server
# Exit screen with Ctrl+A then D

# Print logs on the main screen
cat server.log
# Follow logs on the main screen
tail -f server.log

# To reenter the screen
screen -r server
# To kill the screen
screen -X -S server quit
```

### Create Release Tag
```bash
git tag -a tag-name -m 'tag-message'

git push origin tag-name

# Go to GitHub and create a release
```

### Heroku
```shell
# remote server endpoint
https://consciousness-archive-483dcd2b5c76.herokuapp.com
```


### Set Up Square Subscription
TODO


### TODO
<h4 style="color: red"> High </h4>
  - GCP load balancer for CDN
  - Subscription API
  - Async database read/write?

<h4 style="color: orange"> Medium </h4>
- Admin upload dashboard
  - input article/course as .enex file from Evernote, auto conver to .md
  - input image file, upload to google cloud storage, return url
  - input article/course title
  - upsert to database