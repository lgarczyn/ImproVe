# ImproVe: A Scientific Aid to Improvisation

## Introduction

The goal of this project is to provide beginner musicians with constantly adjusted suggestions.

It can be easily adjusted for any instrument, and adapted to all kinds of consonnance models.

Currently, it gives you a somewhat accurate fretboard-looking suggestion thing.

## Requirements

You need a modern terminal, with true colour support. Apart from that, this should run on anything with enough power.

If you get the "Package alsa was not found in the pkg-config search path." on build,
install 'libasound2-dev'

You also need cargo, but that's a bit of a given for any rust project.

Currently assumes data packet to be 1024 bytes, with 48000 samples every seconds.

### To Do

* Actually read the input format, and remove hard-coded values
* Add controls for input resolution, resampling and fft chunks size
* Try adding auditory suggestions on keypress
* Try adding a better dissonance model, using an audi sample of the targeted instrument
* Keep researching alternatives to FFTs
* Language and Instrument switching
