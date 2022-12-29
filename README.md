# 🍏 Apple Roulette [![Crates.io](https://img.shields.io/crates/v/apple-roulette.svg)](https://crates.io/crates/apple-roulette)

Штрудель?

## Installation
### Mac M1
```
curl -L -o apple-roulette https://github.com/Forestryks/apple-roulette/releases/download/v0.1.0/apple-roulette-aarch64-apple-darwin
chmod +x apple-roulette
sudo mv apple-roulette /usr/local/bin
```
### Linux x86_64
```
curl -L -o apple-roulette https://github.com/Forestryks/apple-roulette/releases/download/v0.1.0/apple-roulette-x86_64-unknown-linux-gnu
chmod +x apple-roulette
sudo mv apple-roulette /usr/local/bin
```

## Usage

Just run `sudo apple-roulette` and hope not to be strudelled.

## A bit of explanation

- Discovers MAC addresses of every host in local network by sending ARP packages to every IP address in subnet
- Searches OUI database for Apple owned prefixes and matches found MACs with them
- If MAC address is not Apple's than it can still be IPhone (IPhone uses random MAC every time it connects to network), so checks 62078 port, it is usually open on IPhone
- If there are more than 20 (`apple-roulette -h` to change) apples in your network, than you are strudelled and your laptop halts

## Caveats

No, double apple doesn't count even as one apple.
