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
