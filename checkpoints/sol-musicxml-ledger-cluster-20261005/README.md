# C4加線付き入力への限定接続

実装ソース `275d851e50c2c543111adfb880d5733bcdb28117`、tree `b70d617bffd3f1ccbfafe08e6160005b6d8a709c`。通常五線・Bravura/線間11px・傾き0・ト音記号・C major・明示4/4・単声。新たに検証した入力はC4 E4 G4 B4 | B4 D5 F5 D5の8四分音符・2小節です。原版5.11の画像→RHYTHMS/PAGE/export成功とnativeの8イベントを比較しました。原版XMLをnative入力にしていません。

同梱実行物はLinux x86_64の省容量debug=0、incremental=false、unoptimizedビルドをstripしたものです。新release全体検証ではありません。既存Cloud Linux（GLIBC2.35）で実行確認。他Linux配布先は未検証。旧版の成功入力とrelease実行物は以前のチェックポイントに保持しています。

```sh
tar -xzf audiveris-musicxml-linux-x86_64.tar.gz
sha256sum -c SHA256SUMS
./audiveris-musicxml examples/ledger-quarter.png ledger-quarter.musicxml ledger-quarter.report.json
cmp ledger-quarter.musicxml expected/ledger-quarter.musicxml
```

出力先は未存在にしてください。Java不要。実音高・音価・小節をnativeの画像認識結果から出力します。

今回の変更はLedgerInterの弱候補除去免除、canonical水平glyph消去、2部品までのGlyphCluster評価、実在加線へのdot距離チェック、既存ClefInter音高式のC4接続です。残りの加線音高・3部品以上のcluster・compound musical Interの競合整理・真正dotのlateChecksは未接続です。中央四分休符の既知誤認は原版も同じ限界のため拒否維持。

対象回帰4件＋cluster境界1件＋foundation境界2件成功、fmt・root lib/bins clippy成功。544全回帰は旧ソースa2bf13caの結果で、今回の全回帰未実行。実スキャン/別フォント/skew/特殊staff/full24形や新拍子・調号・付点へ拡張していません。HEADS/STEMS/MusicXML全体完了フラグfalse。Library結果不明は保持し再探索・再upload無し。
