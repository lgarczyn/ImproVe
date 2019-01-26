# ImproVe: A Scientific Aid to Improvisation

## Introduction

The goal of this project is to provide beginner musicians with constantly adjusted suggestions.

It can be easily adjusted for any instrument, and adapted to all kinds of consonnance models.

Currently, it gives you a somewhat accurate fretboard-looking suggestion thing.

![alt text](https://i.imgur.com/XD9MSTb.png)

## Requirements

You need a modern terminal, with true-colour support. Apart from that, this should run on anything with enough power. The clear-screen signal is currently terminal specific, but not eactly needed.

If you get the "Package alsa was not found in the pkg-config search path." on build,
install 'libasound2-dev'

You also need cargo, but that's a bit of a given for any rust project.

Run with `cargo run`, help with `cargo run -- -h`

The CPAL library can cause some crashes

### To Do

* Stop assuming bitrate
* Maybe add possible inbetweening for powerful computers
* Audio feedback mode ?
* Try adding a better dissonance model, using a sample of the targeted instrument
* other displays than guitar
