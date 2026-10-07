# Original dependency

Source SHA9e1e55cd2746037d059345881c53e6a6754bffbd, official5.11 runtime:

- [AbstractChordInter.getDurationSansTuplet/getEndTime](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/sig/inter/AbstractChordInter.java#L592)
- [Rational reduction/plus/times/toString](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/math/Rational.java#L106)

Existing real saved-owner probe and observation are retained in
`fixtures/official-manual-chord-cache/observed.json`. No new original image run
or coordinate search was needed. Native ordinary getters use exact fractions;
current StackRhythm/export still requires sixteenth ticks. Overflow is rejected,
not emulated as Java int wrapping. Full signed/general Rational parity is pending.
