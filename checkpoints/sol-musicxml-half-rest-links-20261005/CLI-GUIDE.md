# 限定版 画像 → MusicXML Linux CLI

この版は、対応条件を満たす譜面画像を読み、認識した音高・音価・休符・小節・単声の開始時刻を MusicXML 4.0 に出力します。実行時に Java や原版の認識XMLは使いません。以下の入力例と受理制約で区切った版です。Audiveris全機能の移植完了、任意の実譜面への対応完了は意味しません。

## 実行物と確認済みの版

この手順書と同じディレクトリにある `audiveris-musicxml-linux-x86_64.tar.gz` を取得してください。非公開リポジトリへの既存の読取権限が必要です。

| 項目 | 値 |
| --- | --- |
| 実装ソース | `a2bf13ca8da7afe0b9e41da491190558898f4b34` |
| ソースtree | `410d8bf30a826e17316cbaf4cc60a1cff656ea2e` |
| 実装・実行物を保全したmain | `2ec4704f13817ca04a83eac41686a7abb0e2f1cb` |
| 実行環境 | Linux x86_64、GLIBC 2.35以上が必要。libgcc_s/libm/libcを動的使用。既存Cloud環境で確認済み。他ディストリビューションは未検証 |
| ライセンス | AGPL-3.0-or-later。配布物内のLICENSEを参照 |
| tar.gz SHA256 | `94913eff21633791d9e961bcc978395f79afcef4f54bfd98f5e947ac3271eb20` |
| 実行物 SHA256 | `70938e9caf851b10b6cef4962bcb398a8ef9f3b5c14c9815d722b2d4275a82af` |

この手順書の追加では、実装・実行物・元のtar.gzを変更していません。ソース復元手順は同じチェックポイントのMANIFEST.json / RESTORE.py / SOURCE-SHA256.jsonにあります。

## 入力条件

| 項目 | この版の条件 |
| --- | --- |
| ファイル | 不透明なPNGまたはPGM。400万画素以下、ファイル32,000,000 bytes以下。成功確認はPNG |
| 字形・サイズ | Bravura、五線の線間隔11px、傾き0。nativeテンプレート44ptに対応する画像。固定検証rendererでは原版51ptを使用 |
| 五線・音部記号 | 通常の均一な五線、ト音記号 |
| 調・拍子 | C major（調号・臨時記号なし）、画像内に明示された4/4。拍子を省略した画像は推測せず拒否 |
| 音符・休符 | 四分音符・二分音符・四分休符・二分休符。連桁・旗・付点なし |
| パート | 単一の非移調旋律パート、単声。同時発音の和音・複数staffなし |
| 小節・改行 | 各小節が四分音符換算4拍で完結。2小節・4小節・2段への改行を確認 |
| 複数画像 | batchで画像ごとに独立したMusicXMLを作成。複数ページを一曲へ結合する機能ではない |

これらは必要条件です。同じ記号種でも、弱い候補・隣接する複合部品・未対応の接続候補が残る配置は受理できません。任意の配置を保証せず、曖昧な認識は拒否します。

`--part-name Flute` は出力パート名の指定だけです。楽器を画像から識別したり、移調したり、認識条件を変更したりしません。単声・非移調で扱えるパート名を利用してください。名前は空でない1行、UTF-8で256 bytes以下です。

## 最初の実行

専用ディレクトリにtar.gzを置き、その場所で実行します。

```sh
sha256sum audiveris-musicxml-linux-x86_64.tar.gz
tar -xzf audiveris-musicxml-linux-x86_64.tar.gz
sha256sum -c SHA256SUMS
mkdir out
./audiveris-musicxml --output-dir out --part-name Flute \
  examples/quarter-rest.png \
  examples/half-two-measures.png \
  examples/quarter-half-break.png \
  examples/half-rest-y241.png
```

`out/` に各入力名の `.musicxml` と `.report.json` ができます。例えば `quarter-rest.musicxml` と `quarter-rest.report.json` です。成功ごとに標準出力へ `status: written` を含むJSONを1行表示します。

単一画像では出力名も指定できます。この形式の既定パート名は `Music` です。

```sh
./audiveris-musicxml examples/half-rest-y239.png \
  half-rest.musicxml half-rest.report.json
```

出力先ディレクトリは先に作成し、出力ファイルが存在しない場所を選んでください。再実行時は別の出力先を使います。既存の出力は上書きしません。batchの途中で認識に失敗すると停止し、それ以前の成功したMusicXML/reportの組は残ります。失敗した入力の組は出力しません。書込失敗時はその入力の作成済みファイルを削除し、削除にも失敗すればその旨をエラーで知らせます。

`expected/` には `--part-name Flute` で作成した確認済みの出力例があります。上のbatchの出力と比較できます。単一画像形式は既定パート名が異なるため、バイト単位では一致しません。

```sh
cmp out/quarter-rest.musicxml expected/quarter-rest.musicxml
cmp out/half-rest-y241.musicxml expected/half-rest-y241.musicxml
```

配布実行物の `--help` の短い説明には二分休符の記載がありませんが、この版で以下の2位置を確認済みです。詳しい対応範囲は本手順書を参照してください。

## 同梱した入力例

| 入力例 | 内容 | 小節・段 | イベント数 |
| --- | --- | --- | --- |
| quarter-rest.png | 四分音符6、四分休符2 | 2小節、1段 | 8 |
| half-two-measures.png | 二分音符4 | 2小節、1段 | 4 |
| quarter-half-break.png | 四分/二分音符10、四分休符2 | 4小節、2段 | 12 |
| half-rest-y239.png | 二分休符1、四分音符2、二分音符2 | 2小節、1段 | 5 |
| half-rest-y241.png | 同じ二分休符譜面の別位置。到達するheadがないbeam/hook候補を除去 | 2小節、1段 | 5 |

最初の3件は独立した合成譜面24イベントです。最後の2件は同じ二分休符譜面の位置を制御した10イベントであり、独立した実譜面2件ではありません。

四分休符の例の音列は `B4, rest, E4, G4 | B4, rest, F5, D5`（全て1拍）。二分音符の例は `B4, D5 | B4, D5`（全て2拍）。二分休符の例は `rest(2拍), B4(1拍), D5(1拍) | D5(2拍), B4(2拍)` です。

## 既知の受理制約とエラー

確認済みの成功例には、追加修正が必要な既知の不具合はありません。ただし、Bravura/C major/4/4/単声/四分・二分という分類全体の任意配置が完成したとは扱いません。次の再現可能な受理制約が残ります。

| 条件 | 現在の結果・理由 |
| --- | --- |
| central-rest-ambiguous.png（ソースfixture） | `music_filter_weak_head_optionals_pending` で拒否。中央の四分休符配置に生じる弱いhead候補の扱いが未移植。符号種は範囲内だがこの配置は未対応。正しい出力の原版比較を確立していない |
| half-rest-y240.png（ソースfixture） | `musicxml_incomplete_or_excess_measure` で拒否。1px境界で原版の二分休符gradeが0.4834となりLINKSで除去される。nativeも弱閾値0.5で除去し、足りない拍を補造しない。これは保存済み原版との不一致ではない |
| headに届き得るbeam/flag候補 | `music_beam_...` 等で拒否。実際のbeam接続スケジューラは未移植。到達しない孤立候補だけ限定処理 |
| 拍子記号を認識できない | `musicxml_recognized_meter_required_no_default` で拒否。4/4を既定値として補わない |
| 小節が4拍に満たない/超える | `musicxml_incomplete_or_excess_measure` 等で拒否 |
| 曖昧な複合記号・付点候補 | `musicxml_compound_symbol_parts_pending` / `musicxml_possible_dot_lateChecks_pending` 等で拒否 |
| 出力ファイルが既に存在する | 出力先の重複検査で拒否。別ディレクトリまたは別名を指定 |

この版を一区切りとする範囲は、明記した受理条件と確認済み入力例です。中央四分休符の弱候補など一般配置への対応は残作業として保持し、成功例の限定到達点と区別します。

## 未対応・未検証

- 任意の実スキャン・写真・PDF、別フォント/サイズ、非zero skew、cue・打楽器・タブ譜等の特殊staff、24形全ページ伝播。
- 実際の連桁/旗を持つ短い音価、全音符/全休符、臨時記号、付点、tie/slur、連符、多声、和音、弱起、複数staff、移調、複数ページの一曲への連結。
- 弱いoptional候補・一般複合glyph処理、32sources上限を超える選択、原版の全LINKS/reduction/cleaner/epilog。
- 一般的な楽器識別、レイアウト再現、全Linuxディストリビューションでの動作。

画像からの実行経路は実装済みですが、原版との比較境界は「XML保存状態のHEADSから後続工程を原版5.11で実行」です。原版の画像入力から全工程を走らせた一括一致を確認したとは扱いません。MusicXML 4.0 XSD検証、原版proxymusic再読込、実行物を取得し直しての実行も確認しました。

実装ソースa2bf13cで全544回帰、fmt、clippyが成功しています。この文書更新では再ビルド・全回帰の再実行をしていません。旧Laura 352 heads/513 exclusionsはソース47c7d11の履歴証跡であり、現版でLaura全工程を再実行した結果ではありません。HEADS/STEMS/MusicXML/epilog/全自動rasterの全体完了フラグはfalseです。

## 保存状況と引き継ぎ

Gitの実装・Linux実行物の保全は確認済みです。Libraryの今回のソース差分アップロードは成功IDのない `page_reference_upload_outcome_unknown`（再試行不可）です。既存receiptとPage読取を照合しましたが、ファイル一覧の読取ツールがなく、同一原本の参照IDは未特定です。アクセス拒否や確定失敗とは区別し、重複アップロードをしていません。

対象は `Audiveris half-rest image CLI source delta a2bf13c`、61,701 bytes、SHA256 `00434e4092758baa9f289a9ad0bc6d2b0d2203b841a50f81162379ef05751b6a`。既存Libraryの正規Filesでこのファイルの参照IDが確認できれば、同一原本を再取得して照合できます。原本・receiptはGitにも保全済みです。

Libraryの結果不明は未解消の引き継ぎ事項です。テスト削減は着手せず、次の明示指示を待ちます。Mac/GitHub Actions dispatch・rerun/新認証/課金/権限変更は行っていません。

