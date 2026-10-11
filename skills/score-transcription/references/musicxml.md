# MusicXML の書き方（小節を直すときの要点）

`audiveris-native` の出力に合わせた MusicXML 4.0 partwise の最小限の書式。

## 時間

- `<divisions>` は 4 分音符 1 つ分の目盛り数。先頭小節の `<attributes>` にあり、以後同じ値を使う。
  新しい音価（例: 3 連符、64 分）が割り切れないときは、全パート先頭の `divisions` を倍にして全 `duration` も倍にする。
- `<duration>` = 音価 × divisions。付点は 1.5 倍、複付点 1.75 倍、3 連符は 2/3 倍。
  例（divisions=8）: 4分=8、8分=4、付点8分=6、16分=2、32分=1、3連8分=8×½×⅔ → divisions を 12 等にする。
- 各声部の `duration` の合計が拍子の長さ（3/4 で divisions=8 なら 24）に一致すること。弱起は短くてよい。
- 和音の 2 音目以降は `<chord/>` を先頭に置き、同じ `duration` を書く（時間は進まない）。
- 同じ小節で次の声部・五線に移るときは `<backup><duration>小節の長さ</duration></backup>`。
- 見えない埋め草は `<forward><duration>n</duration><voice>v</voice><staff>s</staff></forward>`。
- 装飾音符は `<grace/>`（短前打音は `<grace slash="yes"/>`）で `duration` を書かない。

## 音符の要素の順序（順序を間違えると不正な MusicXML）

```xml
<note>
  <grace/>?  <chord/>?
  <pitch><step>E</step><alter>-1</alter><octave>5</octave></pitch>   <!-- または <rest/> -->
  <duration>4</duration>
  <tie type="start"/>?
  <voice>1</voice>
  <type>eighth</type>  <dot/>*
  <accidental>flat</accidental>?
  <time-modification><actual-notes>3</actual-notes><normal-notes>2</normal-notes></time-modification>?
  <stem>down</stem>?
  <staff>1</staff>?
  <beam number="1">begin</beam>*      <!-- begin / continue / end / forward hook / backward hook -->
  <notations>?
    <tied type="start"/>  <slur number="1" type="start"/>  <tuplet type="start"/>
    <articulations><staccato/><accent/><tenuto/></articulations>
    <ornaments><trill-mark/></ornaments>  <fermata/>  <arpeggiate/>
  </notations>
  <lyric number="1"><syllabic>begin</syllabic><text>Trau</text></lyric>*
</note>
```

- `<alter>` は実音（調号由来も含む）。`<accidental>` は譜面に臨時記号が「見える」ときだけ書く。
- `<type>`: `whole` `half` `quarter` `eighth` `16th` `32nd` `64th`。
- 全小節休符: `<note><rest measure="yes"/><duration>小節の長さ</duration><voice>1</voice></note>`。
- タイは `<tie>`（音）と `<notations><tied>`（表示）の両方。始点 `start`、終点 `stop`、中間は両方。
- 歌詞の `syllabic`: `single`（1 音節語）、`begin`、`middle`、`end`。メリスマの延長線は `<extend/>`。

## 小節の中の他の要素

```xml
<measure number="5">
  <print new-system="yes"/>                       <!-- 段の始まり（OMR が付ける。残す） -->
  <attributes>                                    <!-- 変化するときだけ -->
    <key><fifths>-3</fifths></key>
    <time><beats>6</beats><beat-type>8</beat-type></time>
    <clef number="2"><sign>F</sign><line>4</line></clef>
  </attributes>
  <direction placement="below"><direction-type><dynamics><pp/></dynamics></direction-type><staff>1</staff></direction>
  <direction placement="above"><direction-type><words>rit.</words></direction-type></direction>
  <direction placement="below"><direction-type><wedge type="crescendo"/></direction-type></direction>
  ... notes ...
  <direction><direction-type><wedge type="stop"/></direction-type></direction>
  <barline location="right"><bar-style>light-heavy</bar-style><repeat direction="backward"/></barline>
</measure>
```

- 開始反復: `<barline location="left"><bar-style>heavy-light</bar-style><repeat direction="forward"/></barline>`（小節の先頭）。
- ボルタ: 最初の小節の左 `<barline location="left"><ending number="1" type="start"/></barline>`、
  最後の小節の右 `<ending number="1" type="stop"/>`（脚が無ければ `discontinue`）。
- タイトル等: `<work><work-title>…</work-title></work>`、
  `<identification><creator type="composer">…</creator><creator type="lyricist">…</creator>…</identification>`
  （`identification` 内では `creator` が `encoding` より前）。
