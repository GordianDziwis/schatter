#!/usr/bin/env bash

remote_host=$1

# Set up shell

scp .zshrc .p10k.zsh "$remote_host":
sudo apt install zsh neovim fd-find fzf glances ranger
curl https://pyenv.run | bash
sudo chsh -s "$(which zsh)" pi
sudo update-alternatives --install /usr/bin/vi vi /usr/bin/nvim 60
echo 0 | sudo update-alternatives --config vi
sudo update-alternatives --install /usr/bin/vim vim /usr/bin/nvim 60
echo 0 | sudo update-alternatives --config vim
sudo update-alternatives --install /usr/bin/editor editor /usr/bin/nvim 60
echo 0 | sudo update-alternatives --config editor

# Install rpi-ws281x-rust
