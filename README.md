<p align="center">
  <a href="https://consciousnessarchive.com">
    <img alt="Consciousness Archive" src="./logo.png" />
  </a>
</p>

[//]: # (# Consciousness Archive)



<h1 style="color: #89509b"> Consciousness Archive Backend </h1>
Server to deliver consciousness calibrations, images, videos, audio, article/course markdown files, and more from local memory,
and read and write customer data and payment links from Square API.


<h3 style="color: #FFFAAA"> Initialize Database </h3>

```shell
cargo make reset_database
```

<h3 style="color: #FFFAAA"> Run Server </h3>

```shell
cargo run -r -p server
```


.

.

.


<h3 style="color: #FFFAAA"> Convert Evernote (.enex) to Markdown </h3>

Install here: [evernote2md](https://github.com/wormi4ok/evernote2md)

```shell
brew install evernote2md

scripts/evernote2md.sh --input some_evernote.enex
```


.

.

.



<h3 style="color: #FFFAAA"> Setup Virtual Machine </h3>

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


.

.

.



<h3 style="color: #FFFAAA"> Create Release Tag </h3>

```bash
git tag -a tag-name -m 'tag-message'

git push origin tag-name

# Go to GitHub and create a release
```


.

.

.



<h3 style="color: #FFFAAA"> Heroku Endpoint </h3>

```shell
# remote server endpoint
https://consciousness-archive-483dcd2b5c76.herokuapp.com
```


.

.

.



<h3 style="color: #FFFAAA"> Ngrok Localhost HTTPS Webhook </h3>

Visit [ngrok](https://ngrok.com/) and login.
Navigate to `Your Authtoken` page and paste this command into the terminal:
```shell
ngrok config add-authtoken <token>
```
Start the server with:
```shell
ngrok http --domain=consciousnessarchive.ngrok.dev 3333
```


.

.

.


<h3 style="color: #FFFAAA"> Square Setup: Subscription Catalog  </h3>

Hit `/api/upsert_subscription_catalog` endpoint.
After creating a catalog, use `result.catalog_object.id`
or use `result.catalog_object.subscription_plan_variation_data.subscription_plan_id`
to set as `SQUARE_SUBSCRIPTION_CATALOG_ID` in the `.env` file.