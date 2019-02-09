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

The dependencies is SDL2 and SDL2-ttf (or libsdl2-dev and libsdl2-ttf-dev for linux)

For the terminal display you will need a modern terminal, with true-colour and termcaps support.

You also need cargo, but that's a bit of a given for any rust project.

Run with `cargo run`, help with `cargo run -- -h`

## To Do

### Features

* Audio feedback mode ?
* Use a sample of the targeted instrument
* Change the dissonance half-life
* Change key parameters at runtime
* other displays than guitar
* Better smoothing of dissonance curve over octaves
* Note graph adjusting to max amplitude not current amplitude
* Changing the ratio of discarded frequencies
* Chord display

### Code

* use Clippy to clean bad practice
* apply fmt more rigorously
