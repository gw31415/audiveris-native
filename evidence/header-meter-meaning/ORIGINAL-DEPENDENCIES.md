# Original 5.11 dependency boundary

Pinned source `9e1e55cd2746037d059345881c53e6a6754bffbd`; original jar SHA256 `9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5`. The following small excerpts are from existing original jar bytecode (`javap -p -c`). No original recognition parameters or algorithm were changed. Native runtime does not invoke Java.

`PageRhythm.populateTimeSignatures` uses the selected `AbstractTimeInter.getTimeRational`; its `getValue` supplies each range duration. `processRanges` passes that duration to `StackRhythm.process`. Native presently handles a constant selected meter, integer-quarter durations and a single complete voice. Meter changes, fractional-quarter rhythms and general range processing remain pending.

```text
  public org.audiveris.omr.math.Rational getValue();
    Code:
       0: aload_0
       1: getfield      #13                 // Field den:I
       4: ifne          9
       7: aconst_null
       8: areturn
       9: new           #19                 // class org/audiveris/omr/math/Rational
      12: dup
      13: aload_0
      14: getfield      #7                  // Field num:I
      17: aload_0
      18: getfield      #13                 // Field den:I
      21: invokespecial #21                 // Method org/audiveris/omr/math/Rational."<init>":(II)V
      24: areturn

```

```text
     303: getfield      #162                // Field org/audiveris/omr/sheet/rhythm/PageRhythm$Range.ts:Lorg/audiveris/omr/sig/inter/AbstractTimeInter;
     306: ifnull        357
     309: aload_3
     310: getfield      #162                // Field org/audiveris/omr/sheet/rhythm/PageRhythm$Range.ts:Lorg/audiveris/omr/sig/inter/AbstractTimeInter;
     313: invokevirtual #127                // Method org/audiveris/omr/sig/inter/AbstractTimeInter.getTimeRational:()Lorg/audiveris/omr/score/TimeRational;
     316: astore        4
     318: aload         4
     320: ifnonnull     339
     323: getstatic     #79                 // Field logger:Lorg/slf4j/Logger;
     326: ldc           #166                // String No timeRational value for {}
     328: aload_3
     329: getfield      #162                // Field org/audiveris/omr/sheet/rhythm/PageRhythm$Range.ts:Lorg/audiveris/omr/sig/inter/AbstractTimeInter;
     332: invokeinterface #168,  3          // InterfaceMethod org/slf4j/Logger.info:(Ljava/lang/String;Ljava/lang/Object;)V
     337: iconst_0
     338: ireturn
     339: aload_3
     340: aload         4
     342: putfield      #172                // Field org/audiveris/omr/sheet/rhythm/PageRhythm$Range.timeRational:Lorg/audiveris/omr/score/TimeRational;
     345: aload_3
     346: aload         4
     348: invokevirtual #176                // Method org/audiveris/omr/score/TimeRational.getValue:()Lorg/audiveris/omr/math/Rational;
     351: putfield      #182                // Field org/audiveris/omr/sheet/rhythm/PageRhythm$Range.duration:Lorg/audiveris/omr/math/Rational;
     354: goto          479
     357: iload_2
     358: ifne          479
     361: aload_1
     362: invokevirtual #186                // Method org/audiveris/omr/score/PageRef.getPrecedingInScore:()Lorg/audiveris/omr/score/PageRef;
     365: astore        4
     367: aload         4
     369: ifnull        479
```

```text
     155: invokespecial #259                // Method org/audiveris/omr/sheet/rhythm/StackRhythm."<init>":(Lorg/audiveris/omr/sheet/rhythm/MeasureStack;)V
     158: aload_2
     159: getfield      #182                // Field org/audiveris/omr/sheet/rhythm/PageRhythm$Range.duration:Lorg/audiveris/omr/math/Rational;
     162: invokevirtual #262                // Method org/audiveris/omr/sheet/rhythm/StackRhythm.process:(Lorg/audiveris/omr/math/Rational;)V
     165: goto          195
     168: astore        8
```

The input fixture is an independent original Bravura rendering with six quarter notes in two 3/4 measures. Full-image MusicXML comparison is separate from saved-OMR identity/glyph/grade/measure comparison. Only one new input was selected; no coordinate sweep. Strict rejection of underfilled native measures remains admission policy, not a claim that original MeasureFixer rejects pickups or partial measures.
