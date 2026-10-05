# Original COMMON_TIME dependencies

Pinned original source `9e1e55cd2746037d059345881c53e6a6754bffbd`, same unmodified5.11 jar as prior meter fixture. Bytecode excerpts are from local `javap -p -c`; no network/API or recognition parameter changes.

```text
      29: aload_0
      30: invokevirtual #1020               // Method getHeadersInters:()Ljava/util/List;
      33: astore_3
      34: aload_0
      35: getfield      #23                 // Field sig:Lorg/audiveris/omr/sig/SIGraph;
      38: getstatic     #1023               // Field overlapPredicate:Ljava/util/function/Predicate;
      41: invokevirtual #672                // Method org/audiveris/omr/sig/SIGraph.inters:(Ljava/util/function/Predicate;)Ljava/util/List;
      44: invokeinterface #904,  1          // InterfaceMethod java/util/List.stream:()Ljava/util/stream/Stream;
      49: aload_3
      50: invokedynamic #1027,  0           // InvokeDynamic #11:test:(Ljava/util/List;)Ljava/util/function/Predicate;
      55: invokeinterface #312,  2          // InterfaceMethod java/util/stream/Stream.filter:(Ljava/util/function/Predicate;)Ljava/util/stream/Stream;
      60: invokedynamic #1030,  0           // InvokeDynamic #12:test:()Ljava/util/function/Predicate;
      65: invokeinterface #312,  2          // InterfaceMethod java/util/stream/Stream.filter:(Ljava/util/function/Predicate;)Ljava/util/stream/Stream;
```

```text
  private static boolean lambda$reduce$0(java.util.List, org.audiveris.omr.sig.inter.Inter);
    Code:
       0: aload_0
       1: aload_1
       2: invokeinterface #1143,  2         // InterfaceMethod java/util/List.contains:(Ljava/lang/Object;)Z
       7: ifne          14
      10: iconst_1
      11: goto          15
      14: iconst_0
      15: ireturn
```

```text
      69: lookupswitch  { // 2
                    15: 96
                    16: 107
               default: 118
          }
      96: aload         4
      98: getstatic     #3225               // Field org/audiveris/proxymusic/TimeSymbol.COMMON:Lorg/audiveris/proxymusic/TimeSymbol;
     101: invokevirtual #3231               // Method org/audiveris/proxymusic/Time.setSymbol:(Lorg/audiveris/proxymusic/TimeSymbol;)V
     104: goto          118
     107: aload         4
     109: getstatic     #3235               // Field org/audiveris/proxymusic/TimeSymbol.CUT:Lorg/audiveris/proxymusic/TimeSymbol;
     112: invokevirtual #3231               // Method org/audiveris/proxymusic/Time.setSymbol:(Lorg/audiveris/proxymusic/TimeSymbol;)V
     115: goto          118
     118: aload_0
     119: invokevirtual #931                // Method getAttributes:()Lorg/audiveris/proxymusic/Attributes;
```

`SigReducer.reduce` gathers Staff HEADER Inter identities and excludes them from physical overlap candidates. Native uses the owned selected MusicTime phase/frozen TimeWhole identity for this bounded header case. Other frozen cross-kind symbol conflicts remain pending. `PartwiseBuilder.processTime` sets COMMON/CUT export symbol; the new image comparison is COMMON_TIME only. `PageCleaner.visit(TimeWholeInter)` passes its real glyph to processGlyph, as recorded in the existing PageCleaner bytecode fixture.
