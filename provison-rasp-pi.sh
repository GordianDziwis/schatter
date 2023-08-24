#!/usr/bin/env bash

remote_host=$1

# Set up shell

scp .zshrc .p10k.zsh raspberrypi-1:
sudo apt update
sudo apt install zsh neovim fd-find fzf ranger git
curl https://pyenv.run | bash
sudo update-alternatives --install /usr/bin/vi vi /usr/bin/nvim 60
echo 0 | sudo update-alternatives --config vi
sudo update-alternatives --install /usr/bin/vim vim /usr/bin/nvim 60
echo 0 | sudo update-alternatives --config vim
sudo update-alternatives --install /usr/bin/editor editor /usr/bin/nvim 60
echo 0 | sudo update-alternatives --config editor
zsh -c "exit"
sudo chsh -s "$(which zsh)" pi
# Setup pwm

sudo bash -c "echo 'blacklist snd_bcm2835' >> /etc/modprobe.d/snd-blacklist.conf"

# Setup SPI
sudo bash -c "echo 'dtparam=spi=on' >> /boot/config.txt"
sudo bash -c "echo 'core_freq=250' >> /boot/config.txt"
sudo bash -c "sed -i '$ s/$/ spidev.bufsiz=32768/' /boot/cmdline.txt" 
# pi 4
sudo bash -c "echo 'core_freq=500' >> /boot/config.txt"
sudo bash -c "echo 'core_freq_min=500' >> /boot/config.txt"
