# audiveris-native (omr)

楽譜画像（PNG/JPEG/TIFF/BMP）を認識して MusicXML 4.0 を出力する Rust 製 CLI です。
[Audiveris](https://github.com/Audiveris/audiveris) 5.11 の認識パイプラインと同じ手順・同じ
アルゴリズム（同じ定数・テンプレート・分類器）を、必要な部分だけ素直に実装し直しています。
Java も Audiveris 本体も不要です。

```sh
cargo build --release
./target/release/audiveris-native page1.png page2.png -o score.musicxml
./target/release/audiveris-native -v --debug out/ page.png   # 工程ごとの時間、注釈画像と認識JSON
cargo test --release                                       # 実譜面1ページの end-to-end テスト
tools/evaluate.sh                                           # samples/ の正解との比較
```

複数画像は同じ曲の連続ページとして 1 つの楽譜にまとめます（拍子・調・音部記号はページを跨いで引き継ぎ）。

## 処理の流れ（Audiveris の各 step と対応）

| step | ファイル | 内容 |
|---|---|---|
| BINARY | `binary.rs` | AdaptiveFilter（37×37 窓、`0.7·mean + 0.9·std`） |
| SCALE | `scale.rs` | 縦ランのヒストグラム（線幅、black+white combo による五線間隔、連桁の太さ、符幹幅） |
| GRID | `staff.rs`, `grid.rs` | 細い横ランの「櫛」で五線を検出し、5本共通の滑らかな曲線で頑健にフィット。全高の縦ピークを小節線とし、接続で段（system）、ブレースでパートを構成 |
| HEADERS | `header.rs` | 段頭の音部記号・調号・拍子を分類器で認識（五線消去で割れた記号は隣接部分を結合して再評価） |
| BEAMS / LEDGERS | `beams.rs` | 連桁らしい厚みの画素から平行四辺形をフィット、加線 |
| HEADS | `heads.rs`, `template.rs` | Bravura の符頭を実測サイズでラスタライズし、fore/back/hole キーポイントによる Audiveris と同じテンプレート距離で、線・間・加線の各高さを走査 |
| STEMS / REDUCTION | `stems.rs` | 符頭の左右から符幹を探索。Audiveris の文脈グレード `(1+C)g/(1+Cg)`、尾の最小長 1.75、2度の反対側規則、臨時記号型の偽符頭排除 |
| SYMBOLS | `symbols.rs`, `classifier.rs`, `features.rs` | 認識済み要素を消した残差のグリフ（と近傍の組合せ）を Audiveris 5.11 の学習済みニューラル分類器で分類。旗・休符・臨時記号・付点・音部記号変更・強弱・連符数字など |
| CHORDS / RHYTHMS | `score.rs` | 和音・休符・音価（旗/連桁/付点/連符）、音高（音部記号・調号・小節内臨時記号）、声部とタイムスロット、暗黙の3連符、隠れた空き五線を含むパート対応 |
| export | `musicxml.rs` | MusicXML 4.0 partwise（XSD 検証済み） |

## 精度

正解は MuseScore の原譜（OpenScore Lieder, CC0）です。`tools/compare.py` で、パート・五線ごとの
（音高または休符, 音価）列を整列して一致率（FLAT）を求めます。小節線の取り違えに左右されない指標です。
同じ画像を公式 Audiveris 5.11.0（Linux 版、既定設定）にも通して比較しました。

`samples/real`（実譜面ページ。An Laura / Der Winter は 180dpi、Der Brautschmuck は 180dpi の MuseScore 描画）:

| 曲 | 本実装 | Audiveris 5.11 |
|---|---|---|
| An Laura | 0.980 | 0.869 |
| Der Winter | 0.960 | 0.831 |
| Der Brautschmuck | 0.965 | 0.823 |

調整に一切使っていない検証用コーパス：シューベルト《冬の旅》全22曲（65ページ、MuseScore 4 で 300dpi 描画）:

| 曲 | Audiveris 5.11 | 本実装 |
|---|---|---|
| wr01 | 0.858 | 0.911 |
| wr02 | 0.983 | 0.970 |
| wr03 | 0.322 | 0.945 |
| wr04 | 0.973 | 0.941 |
| wr05 | 0.951 | 0.967 |
| wr06 | 0.971 | 0.962 |
| wr07 | 0.883 | 0.885 |
| wr08 | 0.986 | 0.961 |
| wr09 | 0.950 | 0.901 |
| wr10 | 0.719 | 0.822 |
| wr11 | 0.726 | 0.989 |
| wr12 | 0.875 | 0.887 |
| wr13 | 0.375 | 0.478 |
| wr14 | 0.911 | 0.865 |
| wr15 | 0.931 | 0.872 |
| wr16 | 0.950 | 0.940 |
| wr17 | 0.981 | 0.980 |
| wr18 | 0.808 | 0.945 |
| wr19 | 0.953 | 0.935 |
| wr20 | 0.988 | 0.951 |
| wr21 | 0.934 | 0.892 |
| wr22 | 0.977 | 0.983 |
| 平均 | 0.864 | 0.908 |

（wr18 は Audiveris が 6 ページ中 1 ページを処理できませんでした。）
処理時間は 300dpi の 1 ページで約 5 秒（1 スレッド）です。

コーパスの再現: OpenScore Lieder の `.mscx` を MuseScore で
`mscore -r 300 -o NAME.png NAME.mscx`（`NAME-1.png, NAME-2.png, …` が出力される）とし、
`tools/evaluate.sh コーパスのディレクトリ` を実行します。

## 対応していないもの

- タイ・スラー・歌詞・テキスト・アーティキュレーションの出力（強弱記号のみ出力）
- 装飾音符の出力（誤って通常の音符にしないよう除外するだけ）
- 写真・傾きの大きいスキャン、タブ譜・打楽器譜、PDF 入力
- 連桁の束が五線の線とちょうど重なる箇所、一部フォントの16分音符の旗

## ライセンス・由来

AGPL-3.0-or-later（Audiveris に準拠）。`assets/classifier.json` は Audiveris 5.11.0 の学習済みモデル、
`assets/Bravura.otf` は SIL Open Font License（`assets/BRAVURA-OFL.txt`）。
`src/features.rs` と `src/labels.rs` は以前の検証済み移植（Audiveris の MixGlyphDescriptor / モデル定義）を
そのまま利用しています。`samples/real/*.mscx` は OpenScore Lieder（CC0）。
