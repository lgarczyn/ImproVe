# ImproVe: A Scientific Aid to Improvisation

## Introduction

The goal of this project is to provide beginner musicians with constantly adjusted suggestions.

It can be easily adjusted for any instrument, and adapted to all kinds of consonance models.

Currently, it gives you a somewhat accurate fretboard-looking suggestion thing.

![alt text](https://i.imgur.com/XD9MSTb.png)

## Notes

Because the analysis is (ideally) objective, by estimating sound roughness, the suggestions do not take into account any cultural parts of perceived consonance and dissonance. This could potentially help a musician to break out of habits and explore music outside of trained norms.

On the other hand, because the analysis is still a couple levels of abstractions removed from the actual perception of dissonance,
a lot of information is lost, and up to the composer to estimate, by instinct or traditional music theory.

## Requirements

The dependencies is SDL2 and SDL2-ttf (or libsdl2-dev and libsdl2-ttf-dev for linux)

For the terminal display you will need a modern terminal, with true-colour and termcaps support.

You also need cargo, but that's a bit of a given for any rust project.

Run with `cargo run`, help with `cargo run -- -h`

If experiencing lag, consider `cargo run --release` and the `-o` option, which allows the program to 'skip' audio data.

## To Do

### Features

* Audio feedback mode ?
* Change parameters at runtime
* Other displays than guitar
* Better smoothing of dissonance curve over octaves
* Make the note graph indicate value not just diff to other values
* Changing the ratio of discarded frequencies
* Chord display
* Put back dissonance exp lookup table to accelerate load time

### Output quality

* Use actual sample of the targeted instrument
* Use a more scientific secondary beatings estimate (dissonance over time)
* Option for a music theory approach instead of the current formula

### Code

* use OCTAVE const for different octave sizes
* rename Frequency to Component, and Frequency.value to Frequency.component
* clean the interval to interval "map" template function, to garantee output range
  * also move it to its own place
  * maybe as a Trait impl ?