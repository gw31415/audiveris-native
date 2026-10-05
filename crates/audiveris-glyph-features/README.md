# Audiveris isolated glyph features (Rust)

**孤立glyphのbinary pixels / canonical runs →110 raw features**を公式5.11.0から実移植した追加crate。既存classifier crateのAPI・ファイルは変更せず、検証時だけpath dev-dependencyとして使用する。production依存はRust標準libraryだけ。page segmentation、五線削除、glyph候補生成、OCRは扱わない。

## 確認した特徴順と依存

原典は `9e1e55cd2746037d059345881c53e6a6754bffbd`。原文は `oracle/upstream/org/audiveris/omr/` に保存。

|出力index|公式処理|順序/意味|
|---|---|---|
|0..98|MixGlyphDescriptor + BasicARTExtractor|ANGULAR=20, RADIAL=5。p昇順→r昇順、(0,0)だけ除外。F001..F194の99個|
|99..108|GeometricMoments先頭10個|weight,width,height,n20,n11,n02,n30,n21,n12,n03|
|109|MixGlyphDescriptor|glyph bounding boxのheight/width|

ARTの前段は `Glyph.getARTMoments → RunTable.computeArtMoments → cumulate → PointsCollector → AbstractExtractor → BasicARTExtractor/BasicLUT`。幾何は同じ点群を `GeometricMoments` へ渡す。**Hu係数やxBar/yBarは110入力に含まれない**。

細部:

- RunTable.cumulateのno-ROI経路はsequence位置昇順・run開始位置昇順、**各run内のpixelは末尾→先頭**。水平/垂直で点の走査順が変わる。
- ART中心はJava intの座標和、radiusは中心からの最大|dx|と最大|dy|のhypot。最大ユークリッド距離へ置換しない。
- LUT半径50、サイズ101×101、20×5×実虚の計200面。単位円の**内側 rad<1**だけに値を置き、境界/外は0。bilinear補間後に質量で割りhypotを取る。
- Java `Double.MIN_VALUE` は最小正subnormal。単一pixelのradius処理でRust `MIN_POSITIVE`と混同しない。
- 幾何weightはmass/interline²、幅・高さはforegroundのinclusive extent/interline。2次central momentsはmass²、3次はsqrt(mass⁵)で正規化。逆順ループと積の結合順を保持。
- aspectだけは指定bounding boxの幅・高さ。paddingを勝手にtrimしない。

一次資料: [MixGlyphDescriptor](https://github.com/Audiveris/audiveris/blob/5.11.0/app/src/main/java/org/audiveris/omr/classifier/MixGlyphDescriptor.java)、[BasicARTExtractor](https://github.com/Audiveris/audiveris/blob/5.11.0/app/src/main/java/org/audiveris/omr/moments/BasicARTExtractor.java)、[RunTable](https://github.com/Audiveris/audiveris/blob/5.11.0/app/src/main/java/org/audiveris/omr/run/RunTable.java)。

## APIと範囲

`Geometry { width, height, left, top, interline, orientation }` と、row-majorのbool pixels (`true=foreground`) または `Run { position, start, length }` を渡す。

```rust
let raw: [f64; 110] = audiveris_glyph_features::from_bitmap(geometry, &pixels)?;
// または from_runs(geometry, &canonical_runs)
let scores = classifier_model.evaluate(&raw)?.scores;
```

runsは位置/開始順で、同一sequence内に重複・隣接を残さない最大run表現。unsorted/adjacent入力はエラー（暗黙にmergeして演算順を変えない）。空glyph、範囲外run、pixel配列長不整合、interline=0を拒否。資源上限はforeground100万pixel。Java int座標和overflowやinterline² overflow領域も明示エラー。これら不正/overflow入力でJavaのwrapを再現することはしない。

LUTはOnceLockで共有し約16.3MB。現在は正確性優先でforeground点を展開する。最適化やpage処理の移植完了を主張しない。

## 実Java goldenと結果

`tests/fixtures/java-glyph-golden.json` は公式Javaの実行結果。2つの独立JVMでbyte同一。

SHA256: `49fdc7c89af9df04879759b5892534afd66495e1fa46fae8279ad48a9ffafbd3`

96ケース = 合成9形状（楕円・ring・stem・cross・diagonal・2block・single pixel・row・column）とBravura 3記号 × interline 8/16 × offset 0/37 × HORIZONTAL/VERTICAL。

BravuraのG clef U+E050、flat U+E260、black notehead U+E0A4は公式BravuraSymbolsでcodepoint確認。固定フォントのname tableに**2021 Steinberg copyright、Version1.392、OFL-1.1全文**を直接確認し、元fontとその文を保存。Java AWTのheadless画像bufferで生成、UI未起動。fixtureはbitmapとして固定され、Rust試験にfont rendererは不要。

oracleは実 `Glyph`, `RunTable`, `PointsCollector`, `MixGlyphDescriptor`, moments各クラスを5.11.0原文のままコンパイルして使用。旧5.5.3 Audiveris jarはcompile-timeの未実行参照解決だけで、**runtimeには含めない**。class-load logで確認済み。Constant/ConstantSetの2つだけはtest stubで、ARTのprintWatch=falseとconfiguration登録を代替。数値処理・点群列挙はstub化していない。

固定許容差（初回比較前に設定、変更なし）:
`abs(actual-reference) <= 1e-12 + 1e-10*abs(reference)`。

|比較stage|値数|最大絶対差（debug/release同じ）|
|---|---:|---:|
|raw110特徴|10,560|3.0531133177191805e-16|
|Rust特徴からの正規化|10,560|7.993605777301127e-15|
|同中間層|14,304|3.885780586188048e-15|
|同最終score|14,304|1.4432899320127035e-15|
|Java特徴を入力した推論単独|14,304|3.885780586188048e-16|

計64,032値合格。bitmap経路とrun経路も全件exact一致。特徴抽出誤差と推論単独誤差を分けて報告している。これは認識精度評価やpage全体の互換性確認ではない。

```sh
GLYPH_REPORT=verification/golden-debug.json cargo test --locked --offline -- --nocapture
GLYPH_REPORT=verification/golden-release.json cargo test --locked --offline --release -- --nocapture
cargo clippy --locked --offline --all-targets -- -D warnings
```

Java再生成は `python3 oracle/run.py`。既存Audiveris.appのJRE21.0.7・補助jarと、前段oracleで用意した小型compilerを使う。compilerは既定 `../oracle/tools`、`ORACLE_TOOLS`で変更可。生成値が固定fixtureとbyte一致することまで確認する。新しいJDKやGUIは不要。

## 保存・ライセンス

AGPL-3.0-or-later、Audiveris copyright、原著者、変更日をLICENSE/NOTICE/ソースに保持。fontは別のOFL-1.1で、AGPLへ付け替えない。test用classifier modelの既存provenance未確定事項は解消したと主張しない。外部公開なし。

`ARTIFACT-MANIFEST.json` にソース・fixture・font・license・検証結果のSHA256を保存。既存classifier crateやleadのpage処理は変更していない。
