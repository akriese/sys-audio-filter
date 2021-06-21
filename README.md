##### System Audio Filter

A tool to manipulate your system audio.


### TODO:

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



# Dependencies

On Linux, the usage of rodio and cpal requires the ALSA development files.
