# Tabs

Totally accurate band simulator to be.

Early experiment to explore:

- charting notation for various instruments
- polyphonic note/pitch detection from analog signals (string instruments to start out with)
- streaming server support?

## Key technologies used

- `egui`
- `spectrum-analyzer`
- `cpal`
- `rodio`
- `guitarpro`

## So far so good

- Library support for notes and fret notes and tunings along with conversions.
- Initial layout of fret board based on mathematical model.
- Initial layout of fret chart (tab view), with left <- right scrolling of notes.
- Initial fft support
- Real time audio capture + fft + egui
- Initial support for converting between Guitar Pro format (`.gp3`, `.gp4`, `.gp5`) tabulature and the internal representation of a song chart using (`guitarpro`)[https://github.com/slundi/guitarpro]
- DSP functionality to generate matching filter(s) and perform matching.

## Examples

### `audio_playback`

Super simple proof of concept for audio file playback using `rodio`. Supports `.mp3`, `.flac`, `.wav` formats. A file can be provided using the `--path` flag, which defaults to the bundled (Landskap)[] song `A Nameless Fool` in an `.mp3` format.

### `cpal_audio_in`

Graphical spectrum analyzer as an example of on-line FFT. Uses the default input sound device.

### `fret_board`

Rendering example of a guitar fretboard.

### `fret_chart`

Rendering example of a `chart` (notes over time). The default implementation of `FretChart` is devoid of notes, this example reflects that and can serve as a scaffolding.

### `fret_chart_gp`

Extension of the `fret_chart` example, populates the `FretChart` with notes parsed from a Guitar Pro format tabulature file passed through the `--path` flag.

## Some notes on pitch detection

It is in general a challenging task to detect notes (on/off/bending/tremolos etc.) from audio signals. The current approach taken leverages FFT.

The main idea is to provide sufficient resolution to determine energy in corresponding bands. As (in the future) we want to support various stringed instruments (basses as well as guitars with different tunings, we want the design to be sufficiently flexible to cope with the problem).

For a standard tuning of a (6 string guitar) the fundamental frequency of (low) E is 82.41 Hz, the (low) E on a (4 string bass) is one octave below, thus half the frequency (41.2). The shortest interval in Hz between two notes is at the lowest note.

A frequency resolution (binning) of 1 Hz seems reasonable target able to distinguish E at 41.2 from E# at 43.7. Notice, audio signal won't be perfectly pitched so we'll likely have to be a be lenient and taking neighboring bins into account during classification.

Incoming audio signal are typically captured by some audio interface with a sample rate of 41.1kHz, 48kHz, 96kH etc. Using `cpal` we open an audio stream (for now a single channel f32), and forward incoming data to the graphical application (see, `examples/cpal_audio_in.rs`), over a SPSC queue. The gui runs at vsync (typically 60Hz), which on each frame reads the buffered data points. At 60Hz, we will run each 16 ms, and given a sample rate of 48kHz, we will get some 800 samples on average. Incoming data is buffered in a circular buffer, and re-aligned (such that most recent sample is present at the last position of the FFT buffer).

An alternative approach to note detection (pitch and time), is explored in the `octave` folder (see `octave/README.md`).

In `examples/freq_match.rs` the octave code is re-implemented in Rust. `examples/freq_match_using_lib.rs` leverages the `tabs::dsp` lib, providing functions to generate matching filters and apply convolution.

Filter creation on an old(isch) laptop amounts to less than 0.5ms in the worst case (low pitched string with high frequency resolution). Filter generation complexity scales roughly linear to resolution and inversive to pitch (thus overtones/harmonics are less expensive to compute). The convolution operation itself is very time efficient (even if the implementation is completely naive/un-optimized), with a worst case of around 25 us. Thus we, the idea to iterate over a set of harmonics in order to detect a note seems feasible.

Since a tab provides only a fixed number of unique notes (with corresponding harmonics), one might chose to pre-render filters and store in a filter bank cache. It remains to be seen if filter generation becomes a bottleneck, and if so if caching filters is helpful (cache locality versus computations). The filter generation per-se is naively implemented and carries ample opportunities for optimizations.
