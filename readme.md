# ImproVe: A Scientific Aid to Improvisation

## Introduction

The goal of this project is to provide beginner musicians with constantly adjusted suggestions.

It can be easily adjusted for any instrument, and adapted to all kinds of consonance models.

Currently, it gives you a somewhat accurate fretboard-looking suggestion thing.

![alt text](https://i.imgur.com/XD9MSTb.png)

## Notes

Because the analysis is (ideally) objective, by estimating sound roughness, the suggestions do not take into account any cultural parts of perceived consonance and dissonance. This could potentially help a musician to break out of habits and explore music outside of trained norms.

On the other hand, because the analysis is still a couple levels of abstractions removed from the actual perception of dissonance, and cannot currently estimate "second-order beatings", or the dissonance caused by notes played before the current sample, a lot of information is lost, and up to the composer to estimate, by instinct or traditional music theory.

## Requirements

You need a modern terminal, with true-colour support. Apart from that, this should run on anything with enough power. The clear-screen signal is currently terminal specific, but not exactly needed.

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
* different display modes with matching features
