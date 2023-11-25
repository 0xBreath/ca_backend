<p align="center">
  <a href="https://consciousnessarchive.com">
    <img alt="Consciousness Archive" src="./logo.png" />
  </a>
</p>

[//]: # (# Consciousness Archive)


# Consciousness Archive Server
Server to deliver consciousness calibrations, images, videos, audio, article/course markdown files, and more from local memory,
and read and write customer data and payment links from Square API.


### Initialize Database
```shell
cargo make reset_database
```

## Run Server
```shell
cargo run -r -p server
```

## Run Admin
```shell
cargo run -r -p admin -t <file_type> -f <path> -n <name> -i <image_url>
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


### Google Cloud Storage Authentication
Follow this guide if not setup
[Instructions](https://cloud.google.com/sdk/docs/install)
```shell
gcloud auth login
```
See this section: [Guide](https://cloud.google.com/sdk/docs/authorizing#key)
Go to [Service Account](https://console.cloud.google.com/projectselector2/iam-admin/serviceaccounts?supportedpurview=project) as this suggests.
Choose `Consciousness Archive`.
Click on `Actions` (three dots).
Click `Manage Keys`.
Add a new key and download the JSON file.
Copy contents of that JSON into `gcloud_credentials.json`
Make sure `GOOGLE_APPLICATION_CREDENTIALS=./gcloud_credentials.json` is in the `.env` file.


### Ngrok Localhost HTTPS Webhook
Visit [ngrok](https://ngrok.com/) and login.
Navigate to `Your Authtoken` page and paste this command into the terminal:
```shell
ngrok config add-authtoken <token>
```
Start the server with:
```shell
ngrok http --domain=consciousnessarchive.dev.api 3333

```
Set the `Forwarding` address as `WEBHOOK_URL` in the `.env` file.


# Admin Setup for Square API

### Create Square Subscription Catalog
Hit `/api/upsert_subscription_catalog` endpoint.
After creating a catalog, use `result.catalog_object.id`
or use `result.catalog_object.subscription_plan_variation_data.subscription_plan_id`
to set as `SQUARE_SUBSCRIPTION_CATALOG_ID` in the `.env` file.


### Create Square Coaching Catalog
Hit `/api/upsert_coaching_catalog` endpoint.
After creating a catalog, use `result.catalog_object.id`
to set as the `SQUARE_COACHING_CATALOG_ID` in the `.env` file.


### Create Custom Attributes for Customers
Hit `/api/create_attributes` endpoint.


## TODO
- [ ] Load database into Mutex<HashMap> for async access