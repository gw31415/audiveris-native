---
name: score-transcription
description: >-
  Transcribe sheet-music images (PNG/JPEG/TIFF/BMP, or PDF pages rendered to images) into
  MusicXML with a vision model. The audiveris-native OMR CLI produces a first draft that is
  used only as a reference; the image is the source of truth, and every measure is checked
  and corrected against it. Use when the user asks to transcribe, digitize, or convert a
  score / 楽譜 image to MusicXML, or to proofread / 校正 an OMR (Audiveris) result.
  Works with a local CLI or a CLI on a remote host reached through a shell MCP such as
  dev-session-mcp.
---

# 楽譜画像 → MusicXML（VLM 校正）

**正本は画像、`audiveris-native` の出力は参考データ。** OMR の出力は高精度でも誤検出・見落としを含む
（下の「OMR の弱点」）。OMR の音符列をそのまま信じて書き写さず、全小節を画像で見て確かめてから採用する。
OMR の出力は「小節の区切り・音高の目安・拍の骨組み」を早く得るための下書きとして使う。

`scripts/scoretool.py`（Python 3 標準ライブラリのみ。切り出しは Pillow か ImageMagick）を使う。
以下 `$SKILL` はこの SKILL.md のあるディレクトリ。

## 0. 実行環境を決める

- **ローカル**: `audiveris-native --help` が通ればそのまま。無ければ導入（[references/install.md](references/install.md)）。
- **リモート（dev-session-mcp などシェル系 MCP 経由）**: CLI が VPS 等にある場合。
  手順は [references/remote.md](references/remote.md)。以下の各コマンドはリモートで実行し、
  画像は `get_image`、テキストは実行の標準出力で受け取る。
- どちらも無理なら、CLI 無しで画像だけから書き起こす（手順 3 以降を、参考データ無しで行う）。

入力が PDF なら先に画像化する: `pdftoppm -r 300 -png score.pdf page`（300dpi 推奨。150dpi 未満は精度が落ちる）。

## 1. OMR の下書きを作る

```sh
mkdir -p work
audiveris-native --debug work/debug -o work/omr.musicxml page-1.png page-2.png   # ページ順に全部
# 歌詞を読むなら --lang（Tesseract が必要。例: --lang deu, --lang ita, --lang jpn）
```

- 複数ページは同じ曲の連続ページとして 1 ファイルになる。別の曲は別に実行する。
- `work/debug/<画像名>.png` は認識結果の注釈画像（赤=黒符頭、青=白符頭、緑枠=小節、水色=記号）。
  OMR が何を見落としたかを素早く見るのに使う。

## 2. 照合用の材料を作る

```sh
python3 $SKILL/scripts/scoretool.py crops work/debug work/crops page-1.png page-2.png   # 段ごと＋2小節ごとの拡大
python3 $SKILL/scripts/scoretool.py summary work/omr.musicxml > work/omr.txt          # 小節ごとの要約
python3 $SKILL/scripts/scoretool.py check work/omr.musicxml                            # 拍の過不足など
```

- `crops` は段 `S01.png` と、2 小節ずつ拡大した `S01-1-2.png`… を作る（`--split N` で小節数、
  `--split 0` で段のみ）。`index.json` に段→ページ・検出小節数。
- `summary` の各小節見出しの `S<n>` が段番号で、`S<n>.png` と対応する。記法は
  [references/notation.md](references/notation.md)。
- `check` の「overfull/underfull」は音価や声部の誤り、拍子の誤認の強い手がかり。

## 3. 段ごとに画像と照合する（本作業）

まずページ全体を 1 回見て、曲の構成（パート数・各パートの五線数・調・拍子・反復/ボルタ・
ページの段数）を把握し、OMR の構成（`summary` 冒頭、`check` の小節数）と食い違わないか確かめる。
構成が大きく違う（パート数・段の小節数がずれる）なら、その部分は OMR を捨てて画像から書き起こす。

次に段ごと（`S01`, `S02`, …）に繰り返す:

1. `S<n>.png` を見て段全体、`S<n>-a-b.png` で小節を拡大して見る。
2. その段の `summary` の行と、パート・五線・声部ごとに、小節単位で突き合わせる。確認順:
   小節線の数 → 音部記号・調号・拍子の変化 → 各音の音高（加線と臨時記号、調号と小節内の臨時記号の持続）
   → 音価（符頭の白黒・旗/連桁の本数・付点・連符）→ 休符 → タイ/スラー → 強弱・発想記号・アーティキュレーション
   → 歌詞。
3. 違いを直す（手順 4）。分からない箇所は推測で埋めず、最ももっともらしい読みを入れて「不確か」として記録する。
4. 直した段について `check` を流し、拍の過不足が無くなったことを確かめてから次の段へ。

注意:
- 画像に無いものを足さない。OMR が出した記号（特にカエスーラ・トリル・アクセント・ボルタ番号）は、
  画像で確認できたものだけ残す。
- 移調楽器・オッターヴァは記譜音高のまま書く（`8va` は direction で表す）。
- ピアノ譜などの多声部は、符幹の向きで声部を分ける（上声 voice 1・下声 voice 2、下の五線は voice 5〜）。
  各声部が小節を満たすこと（足りない部分は休符、見えない休符は `<forward>` でも可）。

## 4. MusicXML を直す

`work/omr.musicxml` を `work/score.musicxml` にコピーしてから直す（OMR の原本は残す）。
小節単位で差し替えるのが安全:

```sh
python3 $SKILL/scripts/scoretool.py extract work/score.musicxml --part P1 --measures 5   # 元の XML を見る
# 直した <measure> を fix.xml に書いて
python3 $SKILL/scripts/scoretool.py replace work/score.musicxml --part P1 --measure 5 fix.xml
python3 $SKILL/scripts/scoretool.py insert  work/score.musicxml --part P1 --after 5 fix.xml   # 見落とした小節
python3 $SKILL/scripts/scoretool.py delete  work/score.musicxml --part P1 --measure 6          # 余分な小節
python3 $SKILL/scripts/scoretool.py renumber work/score.musicxml                              # 挿入/削除の後
```

- `replace`/`insert` は断片が整形式でなければ書き込まない。小節の追加・削除は全パートで揃える。
- MusicXML の書き方（divisions と duration、和音・backup、タイ・スラー、歌詞、強弱）は
  [references/musicxml.md](references/musicxml.md)。`divisions` は先頭小節の値を使い続ける。
- タイトル・作曲者・パート名（`<work-title>`, `<creator>`, `<part-name>`）も画像から直す。
  OMR は既定でタイトルに画像ファイル名を、パート名に `Part 1` を入れる。

## 5. 仕上げと報告

1. `check` が 0 件（弱起・最終小節の underfull は画像で確かめた上で許容）。
2. 最後にもう一度全段を通して見て、直し漏れが無いか確かめる。
3. 成果物は `work/score.musicxml`（ユーザーの指定があればそこ）。ユーザーへの報告には次を含める:
   - 直した内容の要約（例: 「12 小節で音価、5 小節で音高、2 小節の欠落を補った、歌詞を全て入力」）
   - 不確かな箇所（小節番号・パート・理由）
   - 表現していないもの（例: 運指、ペダルの細部）
   - OMR がほぼ使えなかった部分があればその旨

## OMR の弱点（重点的に疑うところ）

`audiveris-native` の評価（README）からの傾向。精度の低いものほど OMR の出力を疑う。

| 項目 | 傾向 |
|---|---|
| 歌詞 | Tesseract が無いと出ない。あっても誤読・音節の割当てずれがある。画像から書き直す前提で |
| 発想記号・テンポ・タイトル | OCR 依存。3 文字未満の語は出ない。誤読が多い |
| 装飾記号（トリル・ターン・モルデント） | 適合率 0.2 程度。誤検出が多い |
| タイ | 再現率・適合率とも 0.7〜0.8。スラーとの取り違え、同音のタイの欠落 |
| アルペジオ・フェルマータ | 見落としが多い |
| 装飾音符 | 半分程度は音価・接続が誤り |
| ボルタ | 番号は連番で推定（「1.2.」などは誤る） |
| 小節線 | 小節の結合・分割の誤り。弱起の判定 |
| 拍子 | 段の全五線に拍子記号が見えないと出ない（パートによって `<time>` が欠けることがある） |
| 暗黙の 3 連符・声部 | 小節が溢れる場合の推定。多声部の声部割当て |
| 連桁の多い速い音型・密な和音 | 音価（連桁の本数）と和音構成音の欠落 |
| 小さい五線（オッシア）・1 線譜・打楽器譜・タブ譜 | 非対応。画像から書き起こす |
| 画質 | 低解像度・傾き・かすれ・手書きでは全体に精度が落ちる |
