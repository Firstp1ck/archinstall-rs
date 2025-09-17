# First Option:

## On the live ISO, install rust and git
pacman -Sy --needed rust git

## Clone the repository
git clone https://github.com/<you>/archinstall-rs.git
cd archinstall-rs
cargo run --release

# Second Option:

## Install wget
pacman -Sy --needed wget

## Download the latest release
wget https://github.com/<you>/archinstall-rs/releases/latest/download/archinstall-rs
chmod +x archinstall-rs

## Run the installer
./archinstall-rs