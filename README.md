# System Audio Filter

A tool to manipulate your system audio.

## What does it do?

This programm takes an audio source as input and puts the audio back to an output device
while filtering chosen frequencies. Currently, the combination of a LowPass and a
HighPass filter by [biquad](https://docs.rs/biquad/0.4.1/biquad/)
is being used to accomplish the desired effects.

While running, the filters' frequencies can be modified by commands in the terminal:
```
l<Enter> --> reset the lowpass cutoff to 20khz
l200<Enter> --> set the lowpass cutoff to 200hz
l+400<Enter> --> increase the lowpass cutoff by 400hz
l-150<Enter> --> decrease the lowpass cutoff by 150hz
```
All those work for the HighPass cutoff analogous with `h` as prefix.

The program can be quit by entering `q<Enter>` or using Ctrl+C.


## Setup / Usage

### Windows
Generally,
```powershell
cargo run
```
should suffice if you just want to use a microphone as input. If you want to pipe all your
system's sound to the program you might want to install a virtual audio device such as the
[Virtual Audio Cable](https://vb-audio.com/Cable/). After installing such a device, you need to set the standard
audio output of Windows to that device so that the sys-audio-filter can modify it.


## Dependencies

### Linux
The usage of rodio and cpal requires the ALSA development files.

### Windows
No further requirements except the optional virtual audio cable mentioned above.


## TODO:

- [x] Locate fitting crate to access sound
- [x] Send simple audio (e.g. sine wave) to output
- [x] Receive system audio and send it back to output
- [x] Suppress system audio to prevent double output
- [x] Access system audio on multiple platforms
- [x] Manipulate stream's master volume
- [ ] Manipulate chosen frequencies
- [ ] Convert between different sample rates
- [ ] Command Line interface
  - [ ] Enumerate all available devices and let the user choose input and output
  - [ ] Record into file
- [ ] GUI interface
  - [ ] Live output graph of frequencies
  - [ ] Drag filters in graph

