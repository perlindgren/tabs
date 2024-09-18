# Tabs

Totally accurate band simulator to be.

Early experiment to explore:

- charting notation for various instruments
- polyphonic note/pitch detection from analog signals (string instruments to start out with)
- streaming server support

## Key technologies used

- `egui`
- `realfft`
- `cpal`

## So far so good...

- library support for notes and fret notes and tunings along with conversions.
- Initial layout of fret board based on mathematical model.
- Initial layout of fret chart (tab view), with left <- right scrolling of notes.
- Initial fft support
- Real time audio capture + fft + egui

## Some notes on pitch detection

It is in general a challenging task to detect notes (on/off/bending/tremolos etc.) from audio signals. The current approach taken leverages FFT.

The main idea is to provide sufficient resolution to determine energy in corresponding bands. As (in the future) we want to support various stringed instruments (basses as well as guitars with different tunings, we want the design to be sufficiently flexible to cope with the problem).

For a standard tuning of a (6 string guitar) the fundamental frequency of (low) E is 82.41 Hz, the (low) E on a (4 string bass) is one octave below, thus half the frequency (41.2). The shortest interval in Hz between two notes is at the lowest note.

A frequency resolution (binning) of 1 Hz seems reasonable target able to distinguish E at 41.2 from E# at 43.7. Notice, audio signal won't be perfectly pitched so we'll likely have to be a be lenient and taking neighboring bins into account during classification.

Incoming audio signal are typically captured by some audio interface with a sample rate of 41.1kHz, 48kHz, 96kH etc. Using `cpal` we open an audio stream (for now a single channel f32), and forward incoming data to the graphical application (see, `examples/cpal2.rs`), over a SPSC queue. The gui runs at vsync (typically 60Hz), which on each frame reads the buffered data points. At 60Hz, we will run each 16 ms, and given a sample rate of 48kHz, we will get some 800 samples on average. Incoming data is buffered in a circular buffer, and re-aligned (such that most recent sample is present at the last position of the FFT buffer).
