# 複合四分休符の画像CLI checkpoint

実装ソース `c3451645cdfad9dc3e1dd28d175099f5119ef1b9`、tree `16f5d1918803d1891d3675a1adcfae8438f4d8b3`。今回の入力は既存の成功四分休符画像に1pxの墨切れを一つ加えたものです。原版5.11の画像→RHYTHMS/PAGE/exportとnativeのB4 rest E4 G4 | B4 rest F5 D5、8四分イベントが一致しました。原版認識XMLをnativeの入力にはしていません。

最終目標はUI表示を除く原版の認識・解析・補正・保存・MusicXML出力機能同等性です。現在の通常五線・Bravura/線間11px・zero skew・ト音記号・C major・4/4・単声・四分/二分音符と休符等の制限は、小段階の検証範囲です。最終機能の省略ではありません。

```sh
tar -xzf audiveris-musicxml-linux-x86_64.tar.gz
sha256sum -c SHA256SUMS
./audiveris-musicxml examples/compound-quarter-rest.png compound-quarter-rest.musicxml compound-quarter-rest.report.json
cmp compound-quarter-rest.musicxml expected/compound-quarter-rest.musicxml
```

出力先は未存在にします。Linux x86_64、Cloud GLIBC2.35で実行確認。Java/Cargo不要。省容量unoptimized/debug0/incremental_off/strippedビルドです。他Linux配布先は未検証。旧成功入力/実行物は旧チェックポイントに保持。

2部品のcanonical glyph/union・RestInter/chord/SIG登録を画像経路へ接続しました。Rest/TimeWholeのsupport無し・frozen無しについて、実画素overlapと原版のstrict grade比較/tie時source優先を移植。今回の実ページには競合で落ちる枝がなく、非zero競合は実union/sourcepartを使った原版条件指定3kernelで別途確認しました。saved SYMBOLS XML→実LINKS観測（grade3桁丸め）と、独立画像→MusicXML全工程比較を区別しています。真正dot/全Inter/contextual support/frozen/cross-kind/3部品以上は未接続。

今回の全workspace回帰549件成功、fmt・全workspace/all-targets clippy成功。MusicXML4 schema検証と取得し直した実行物/ソースの復元確認はMANIFEST/receipt参照。全機能完了フラグはfalse。Library不明は再探索/再uploadせず、GHA dispatch/rerun無し。
