# Octave modelling

## Dependencies

Install `octave`, no extra packages needed. Tested with 9.2.0.

## Notes on note detection

Our approach is straightforward to detect/match the occurrence of an expected note by means of matched filters.

There is a trade-off between temporal and frequency resolution (aka. the uncertainty property). This makes it challenging to determine the pitch (fundamental frequency) and time thereof in the incoming signal.

However, in common to all stringed instruments, the resonating string will give rise to harmonics, (over-tones being multiples of the fundamental frequency).

- 1 = 1st harmonic, fundamental
- 2 = 2nd harmonic, one octave
- 3 = 3nd harmonic, one octave and a fifth
- 4 = 4th harmonic, two octaves
- 5 = 5th harmonic, two octaves + a major third
- etc

Regarding pitch detection we can leverage on this property for detecting potential match of by a low latency filter(s) matched to (a set of) harmonics, and (later) validate the fundamental by a higher latency filter.

Harmonic analysis may also help to discriminate between multiple notes played simultaneously (appearing as a summation of the constituents).

Take for example, E2 (low E on a 6 string guitar), and E3 (played one octave above).

The 2nd harmonic of E2 coincides with the 1st harmonic (fundamental) of E3. However the 3rd harmonic of E2, will have not coincide with any harmonic of E3, (but rather with a note played at E3 and a fifth). Thus, robustness/performance can be expected to improve with increasing number of harmonics analyzed.

As an end-user you would want feedback as early as possible to wether you have successfully played a note (or not). To this end, matching harmonics (starting from the highest of interest), will give an early indication that you have likely hit the right note (string/fret), while later being confirmed, harmonic by harmonic.

Notice, the amplitude (and its envelope) of harmonics will differ between each string, fret played, the string material, the frets, the body and the neck as well as pickup(s) and filter settings. Filters also may change the phase of harmonics related to the fundamental.

Never the less, for instruments of interest (guitars/basses) and harmonics for each note played will contribute to the audio signal.

## freq_match

The file `freq_match.m` provides a proof of concept implementation, showing the feasibility of the approach.

The hardest detection problem occurs for the lowest notes, in this case E2 at 82.41 Hz (low E on a 6-string guitar in normal tuning), `f_schk` in code.

We create a matching filter with the number of periods `p = 40` (adjustable). The `sin_cos` filter has a maxima for the expected frequency (`f_sch`). To reduce artifacts, a Hanning window (`p` periods) is applied.

The matching factor is computed by convolving the input signal with the filter.

As an evaluation `data` represents the perfect match, and `data2` the closest miss played note (E2#).

For a period size `p = 40`, the spectral leakage is in worst case less than 20% between the two closest notes playable. For the steady state we have a leakage of less than 20%.

The latency is related to filter window (adjustable by `p`), `ws = round(p * fs / f_sch)`, where `fs` is the sample rate. For this case the steady state latency is 0.5 second, which is fine for validation but will be insufficient to give prompt/real-time feedback.

We can repeat the experiment with `f_ot = 5` (the 6th harmonic). This gives a steady state output with a latency of less than 0.1 second. Thus we can mark the note as likely correctly played much earlier.

To further improve latency, for the candidate selection we could also consider more aggressive period settings, just to check IF there is energy present at all in a lenient fashion. With a period `p = 10`, we get a steady state latency of 0.025s, and if we consider the worst case (regarding spectral leaking) we can determine a preliminary result after 0.012 ms. This is extraordinary, as it meets the criteria of optimal rendering latency for screens running at up to 80 Hz. Notice at these low latencies other parameters such as audio card internal buffering, operating system drivers and such will put limitations. In context of pro-audio setups with small audio buffer sizes enables end-to-end latency well below 10 ms, witch will meet our requirements.

















