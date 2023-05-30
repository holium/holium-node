# Holon
A P2P node that wraps Urbit and gives it scalable properties.

## Getting started

### 1. Setup rust

```zsh
# install rustup
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
# make sure to have c-compiler
xcode-select --install
# Verify installation
rustc --version

# to update rustup for future releases
rustup update
```

### 2. Install tmux

On mac:
```zsh
brew install tmux
```

On linux:
```zsh
sudo apt-get install tmux
```

### 2. Build the project with cargo

Build for dev:
```zsh
cargo build
cargo run
```

Build for production:
```zsh
cargo build --release
```

### 3. Running the node

The below command will print the debug cli
```zsh
cargo run 
```

## tmux guide

### Listing sessions


### Attaching to sessions
```zsh
tmux attach-session -t zod
``` 

### Detaching while in a session

Ctrl + B, let go, then D


## Running in production

### Setup

#### 1. Digital ocean scripts

```zsh
# setup firewall
ufw allow OpenSSH
ufw allow www
ufw allow https
ufw allow 34543/udp
ufw enable
# create and configure user
useradd -s /bin/bash -d /home/{username} -m -G sudo {username}
passwd -d {username}
echo "{username} ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers
# configure ssh keys for user
mkdir -p /home/{username}/.ssh
chmod 700 /home/{username}/.ssh
cp /root/.ssh/authorized_keys /home/{username}/.ssh/authorized_keys
chmod 600 /home/{username}/.ssh/authorized_keys
chown -R {username}:{username} /home/{username}/.ssh
# configure sshd
mkdir -p /etc/ssh/sshd_config.d
cat > /etc/ssh/sshd_config.d/override.conf <<EOF
PermitRootLogin no
PubkeyAuthentication yes
PasswordAuthentication no
EOF
```

#### 2. Linux packages

```zsh
sudo apt-get update
sudo apt-get upgrade
sudo apt-get install -y tmux pkg-config libssl-dev build-essential python3-pip
```

#### 3. Caddy

```zsh
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy


sudo nano /etc/caddy/Caddyfile
```

Paste in the following:

```txt
# Refer to the Caddy docs for more information:
# https://caddyserver.com/docs/caddyfile

# The Caddyfile is an easy way to configure your Caddy web server.
#
# Unless the file starts with a global options block, the first
# uncommented line is always the address of your site.
#
# To use your own domain name (with automatic HTTPS), first make
# sure your domain's A/AAAA DNS records are properly pointed to
# this machine's public IP, then replace ":80" below with your
# domain name.
{
        log
}

node-0.holium.live {
        tls info@holium.com
        reverse_proxy localhost:3030
}

# Refer to the Caddy docs for more information:
# https://caddyserver.com/docs/caddyfile
```

Restart with:

```zsh
sudo systemctl reload caddy
```

### Running

```zsh
cargo build --release
ln -s /target/release/holon holon
ln -s /target/release/node node
```