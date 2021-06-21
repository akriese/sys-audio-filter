##### System Audio Filter

A tool to manipulate your system audio.


### TODO:

- [x] Locate fitting crate to access sound
- [x] Send simple audio (e.g. sine wave) to output
- [ ] Receive system audio and send it back to output
- [ ] Suppress system audio to prevent double output
- [ ] Access system audio on multiple platforms
- [ ] Manipulate stream's master volume
- [ ] Manipulate chosen frequencies
- [ ] Command Line interface
- [ ] GUI interface
  - [ ] Live output graph of frequencies
  - [ ] Drag filters in graph


# Dependencies

On Linux, the usage of rodio and cpal requires the ALSA development files.
