#!/bin/bash
#
# Revoker / A CLI tool for convenient Twitch OAuth token revoking
#
# Copyright (C) 2022 / Mykola "TreuKS"
#
#
# This script makes release binaries for Linux and also Windows, while also stripping them to reduce size.
#
#   It depends on: 
#	- cargo:
#	- 	* x86_64-pc-windows-gnu platform
#   -   * x86_64-unknown-linux-gnu platform (but if you're on linux and you already have Rust, you should have it already)
#   - 7z 
#   - stoml
#   - Default Linux Bash utilities.
#

GRN='\033[0;32m'
BLU='\033[1;34m'
YLW='\033[1;33m'
RST='\033[0m'

# Creates temporary storage directories
mkdir tmp
mkdir tmp/linux
mkdir tmp/windows

# Builds the linux release and copies it into the temporary directory
echo -e "\n\n  ${YLW}Building the linux release.${RST}\n\n"
cargo build --release 
cp target/release/revoker tmp/linux/
echo Linux release built.

# Builds the windows release and copies it into the temporary directory 
echo -e "\n\n  ${BLU}Building the windows release. ${RST}\n\n"
cargo build --release --target=x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/revoker.exe tmp/windows/
echo Windows release built.

# Strips the binaries to reduce size
strip tmp/linux/revoker
strip tmp/windows/revoker.exe
echo -e "\n\n  ${GRN}Binaries have been stripped. ${RST}\n\n"

# Makes the releases directory and parses the version from the Cargo.toml file 
mkdir -p releases

RELEASE_VERSION=`stoml Cargo.toml package.version`
# Generates a random string of characters for the archive name
RANDOM_HASH=`tr -dc A-Za-z0-9 < /dev/urandom | head -c 8 ; echo ''`

# Creates a tar.gz archive with the linux version
cd tmp/linux/
7z a revoker-v$RELEASE_VERSION-x86_64-linux-gnu-$RANDOM_HASH.tar.gz revoker

cd .. && cd ..

# Creates a zip archive with the windows version
cd tmp/windows
7z a revoker-v$RELEASE_VERSION-x86_64-windows-$RANDOM_HASH.zip revoker.exe

cd .. && cd ..

# Copies the release archives to the releases directory
mv tmp/linux/revoker-v$RELEASE_VERSION-x86_64-linux-gnu-$RANDOM_HASH.tar.gz releases/ 
mv tmp/windows/revoker-v$RELEASE_VERSION-x86_64-windows-$RANDOM_HASH.zip releases/ 

# Deletes the temporary folder
rm -rf tmp/

echo -e "\n\n  ${GRN}Done.${RST}\n\n"
