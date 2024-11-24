# (s|c)hatter

Here is a **[video](https://www.instagram.com/p/C7ubAWoMh-_)** of the finished installation.

## What happened

1. Build the monolith with single addressable mapped LEDs
1. A webcam mounted under the ceiling
1. Object track a random visitor
1. Project an [anamorphic circle](https://en.wikipedia.org/wiki/Anamorphosis) on the surface of the monolith for this visitor
1. Provoke some dude to blow it up

## Design

Positions of the LEDs where defined beforehand with an Inkscape [SVG drawing](./public/img-4.jpg). One large side and one small side is driven by one Raspberry Pi. A laptop is running the server components which sends over OSC a vector of RGB8 values to the raspberries, which then drive the leds.

- [Files](https://mega.nz/folder/VhFiTISY#yt0wFpmpHZuciwtbA6dg1w)

## Stack

- Control ws2812b: [smart-leds](https://github.com/smart-leds-rs/smart-leds)
- [Nannou](https://github.com/nannou-org/nannou)

## Install

```bash
cargo install cross
# Nannou
sudo apt-get install curl build-essential python3 cmake pkg-config
sudo apt-get install libasound2-dev
sudo apt install libxcb-shape0-dev libxcb-xfixes0-dev
sudo apt-get install libvulkan1 mesa-vulkan-drivers vulkan-tools
```

![](./public/img-0.jpg)
![](./public/img-4.jpg)
![](./public/img-1.jpg)
![](./public/img-2.jpg)
![](./public/img-3.jpg)
