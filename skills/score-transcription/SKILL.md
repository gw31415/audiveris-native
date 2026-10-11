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

**正本は画像、`audiveris-native` の出力は参考データ。** OMR の出力は誤検出・見落としを含む
（下の「OMR の弱点」）ので、全小節を画像で確かめる。ただし**作業は OMR の下書きを直す形で進め、白紙から
書き起こさない。** 小節の拍が合わない・小節数がずれる・拍子を誤認しているといった不揃いがあっても、
下書きの大半の音高・音価・臨時記号・和音構成は使える。合っている部分はそのまま残し、
違う部分だけを画像に合わせて直す。下書きを丸ごと捨てるのは、その段の音符がほぼ読めていない（音符の大半が
欠けている・別の段の内容になっている）ときだけにし、その場合も段単位にとどめる。

`scripts/scoretool.py`（Python 3 標準ライブラリのみ。切り出しは Pillow か ImageMagick）を使う。
以下 `$SKILL` はこの SKILL.md のあるディレクトリ。
手元の `scoretool.py` に `split`・`merge`・`systems`・`pages` が無い（`invalid choice`）ときはスキルの同梱版が古いので、
audiveris-native リポジトリの `skills/score-transcription/scripts/scoretool.py`（リモートなら `~/audiveris-native/` の同じパス）を使う。

## 0. 実行環境を決める

- **ローカル**: `audiveris-native --help` が通ればそのまま。無ければ導入（[references/install.md](references/install.md)）。
- **リモート（dev-session-mcp などシェル系 MCP 経由）**: CLI が VPS 等にある場合。
  手順は [references/remote.md](references/remote.md)。以下の各コマンドはリモートで実行し、
  画像は `get_image`、テキストは実行の標準出力で受け取る。
- どちらも無理なら、CLI 無しで画像だけから書き起こす（手順 3 以降を、参考データ無しで行う）。

入力が PDF なら、必要なページだけを画像化する。スコアとパート譜が 1 冊になった PDF などで
「バイオリンのパート譜の 1 楽章だけ」のように頼まれたら、まず全ページの縮小一覧を見て範囲を決める:

```sh
python3 $SKILL/scripts/scoretool.py pages score.pdf work/pages.png   # p1, p2, … のラベル付き一覧（スキャンの解像度も表示）
pdftoppm -r 300 -png -f 26 -l 28 score.pdf page                     # 例: 26〜28 ページだけ（page-26.png …）
```

- 楽章の終わりは終止線（複縦線）、次の楽章はテンポ表示（Largo など）と新しい拍子・調で見分ける。
  楽章がページの途中で終わる・始まるなら、そのページの画像を段の境目で切ってから OMR に渡す。
- 300dpi 推奨（150dpi 未満は精度が落ちる）。写真やスキャンの PDF は埋め込み画像の解像度以上にしても細部は増えない。

## 1. OMR の下書きを作る

```sh
mkdir -p work
audiveris-native --debug work/debug --report work/omr.log -o work/omr.musicxml page-1.png page-2.png   # ページ順に全部
# 歌詞を読むなら --lang（Tesseract が必要。例: --lang deu, --lang ita, --lang jpn）
```

- 複数ページは同じ曲の連続ページとして 1 ファイルになる。別の曲は別に実行する。
- `work/omr.log`（`--report`）は認識の報告。最初に読む:
  - `## pages`: ページごとの解像度（interline = 五線の線間隔 px。11 px 未満は警告が出る。精度が全体に落ちるので、
    300dpi 以上で取り直せるならそうする）と、拍子を読めなかった・読み違えたと判断して直した記録
    （例 `no time signature read; 14 of 18 measures last 2/4 -> using 2/4`、
    `time signature read as 2/2 but ... -> using 2/4`、分母が読み分けにくいとき `time signature read as 2/2 or 2/4; ... -> using 2/4`）。これは小節の中身からの推定なので、画像で拍子を確かめる。
  - `## structure`: 小節数・番号の範囲、拍子と調の変化。画像の小節数と比べる。
  - `## flagged measures`: 音価の合計が拍子に合わない小節（と複数小節休符）の一覧。`m12 (page 1, S03): P1 v1
    overfull (5 of 2 quarters) [...]` の形で、小節番号・ページ・段（`S03` は手順 2 の `S03.png`）・
    パート・声部と、[ ] 内に推定原因（小節線の見落とし・余分な小節線・連符・声部割当て）が付く。
- `work/debug/<画像名>.png` は認識結果の注釈画像（赤=黒符頭、青=白符頭、緑枠=小節、水色=記号）。
  OMR が何を見落としたかを素早く見るのに使う。

## 2. 照合用の材料を作る

```sh
python3 $SKILL/scripts/scoretool.py crops work/debug work/crops page-1.png page-2.png   # 段ごと＋2小節ごとの拡大
python3 $SKILL/scripts/scoretool.py summary work/omr.musicxml > work/omr.txt          # 小節ごとの要約
python3 $SKILL/scripts/scoretool.py check work/omr.musicxml                            # 拍の過不足など
```

- `crops` は段 `S01.png` と、2 小節ずつ拡大した `S01-1-2.png`… を作る（`--split N` で小節数、
  `--split 0` で段のみ）。`index.json` に段→ページ・検出小節数。拡大は OMR が検出した小節線で切るので、小節線を見落とした段では
  1 枚に 3 小節以上写る。どの小節かは段全体の `S<n>.png` と段頭の小節番号で確かめる。
- `summary` の各小節見出しの `S<n>` が段番号で、`S<n>.png` と対応する。記法は
  [references/notation.md](references/notation.md)。
- 段の頭に小節番号が印刷されている譜面（パート譜に多い）では、段ごとの小節数を画像と突き合わせる。
  各段の頭の番号（最初の段は弱起なら 0、無ければ 1）を順に並べ、最後の小節番号と一緒に渡す:
  ```sh
  python3 $SKILL/scripts/scoretool.py systems work/omr.musicxml --printed 0,5,10,16,22 --last 26
  ```
  `S03: OMR m9-m13 (5)  image m10-m15 (6)  -1: OMR has 1 measure(s) too few (missed barline); later OMR numbers are off by -2`
  のように、どの段で何小節ずれたか（小節線の見落とし・余分、複数小節休符の数え漏れ）と、OMR の番号が
  画像の番号からいくつずれているかが分かる。段の数そのものが違うと冒頭に注意が出る。
- `check` の「overfull/underfull」は音価や声部の誤り、拍子の誤認の強い手がかり。`check` は声部ごとに数えるので、
  件数は `omr.log` の flagged measures（小節ごと）より多くなる。末尾に拍子の変化の一覧が出る。

### 下書きの使い方を決める

`omr.log` で、どこを重点的に直すかを決める（下書きを使うかどうかではない）。

- **flagged でない小節**: 拍は合っている。画像と照合し、違う音だけ直す（多くはそのまま使える）。
- **flagged の小節**: 拍が合わない。まず原因を画像で見分け、**小節の中身は残したまま**原因の箇所だけ直す。
  - 音価の誤り（旗・連桁の本数、付点、白黒の符頭、連符の見落とし）: その音符の `type`/`duration`/`dot` を直す。
    画質の悪い譜面では「音高の列は合っていて音価だけ違う」小節が多い。音高の列はそのまま使い、音価を画像から付け直す。
  - 音符・休符の欠落や余分（cue 音符・小節番号の数字・装飾音の混入など）: その音符だけ足す・消す。
  - 声部の割れ: 声部番号と `backup` を直す。音高はそのまま使える。
  - [ ] に「barline may have been missed」: 2 小節が 1 つにまとまっている。`split` で分ける（中身は再利用される）。
    音価が信用できないときは `--at` ではなく `--before-note N`（N 番目の音の前で分ける）を使う。
  - 「barline may have been added」: 1 小節が割れている。`merge` でつなぐ。
  - 複数小節休符: 数字（小節番号を読んでいることがある）を画像で確かめ、違えば休符の小節を足す・減らす。
- **拍子の推定・補正の記録がある / 拍子が画像と違う**: 拍子を画像で確かめる。違っていれば `<time>` を直す。
  小節の中身は拍子と独立に読まれているので、拍子を直すだけで多くの小節の拍が合うことがある。
- **小節数が画像と合わない**: 段ごとに画像の小節線と下書きの小節を対応づけ、ずれの起きた箇所だけ
  `split` / `merge` / `insert` / `delete` で合わせる。ずれた箇所より後ろの小節も、対応づけ直せば中身は使える。
  **構成（段ごとの小節数）を全段で先に合わせてから中身を直す**と、以後は OMR の小節と画像の小節が 1 対 1 になり、
  小節番号で照合できる。`systems --printed` のずれが 0 になるまで、前の段から順に直す（直すたびに `renumber`）。
- **誤検出の記号を先に消す**: 下書きの途中の音部記号変更（`clef=F4` など）・フェルマータ・装飾音・coda/segno・
  途中の拍子変更は、画像に無ければ最初に消す。特に誤った音部記号はその後の音高を全部ずらすので、
  消すだけでその段の音高の多くが正しくなる。
- flagged が多くても（2 割を超えても）方針は同じ。直す量が増えるだけで、下書きは捨てない。

## 3. 段ごとに画像と照合する（本作業）

まずページ全体を 1 回見て、曲の構成（パート数・各パートの五線数・調・拍子・反復/ボルタ・
ページの段数）を把握し、OMR の構成（`summary` 冒頭、`omr.log` の structure）と食い違わないか確かめる。
段の小節数がずれていれば、その段で小節を対応づけて `split` / `merge` などで構成を先に合わせる
（中身は残す）。パート数が違う場合だけは、そのパートを画像から書き起こす。

次に段ごと（`S01`, `S02`, …）に繰り返す:

1. `S<n>.png` を見て段全体、`S<n>-a-b.png` で小節を拡大して見る。
2. その段の `summary` の行と、パート・五線・声部ごとに、小節単位で突き合わせる。確認順:
   小節線の数 → 音部記号・調号・拍子の変化 → 各音の音高（加線と臨時記号、調号と小節内の臨時記号の持続）
   → 音価（符頭の白黒・旗/連桁の本数・付点・連符）→ 休符 → タイ/スラー → 強弱・発想記号・アーティキュレーション
   → 歌詞。
3. 違う箇所だけを直す（手順 4）。小節を丸ごと書き直すのは、その小節の音符の大半が違うときだけ。
   分からない箇所は推測で埋めず、最ももっともらしい読み（下書きの読みが妥当ならそれ）を残して「不確か」として記録する。
4. 直した段について `check` を流し、拍の過不足が無くなったことを確かめてから次の段へ。

注意:
- 画像に無いものを足さない。OMR が出した記号（特にカエスーラ・トリル・アクセント・ボルタ番号）は、
  画像で確認できたものだけ残す。
- 移調楽器・オッターヴァは記譜音高のまま書く（`8va` は direction で表す）。
- パート譜の **cue 音符**（「Fl.」などと書かれた小さい音符で、下に休符がある）は他の楽器の手がかりで、このパートは
  休んでいる。既定では小さい音符を書かず、本来の休符（全休符など）だけを書く。ユーザーが cue も欲しいと言えば
  `<cue/>` 付きの音符として別声部に書く。
- **複数小節休符**: 数字の付いた休符の小節は、数字の数だけ休みの小節がある（「1」は 1 小節、「4」は 4 小節）。
  古い版では 2 小節を倍全休符（太い縦の四角）、4 小節を長休符で書くことがある。MusicXML では小節を数字の数だけ作り、
  それぞれ全小節休符（`<rest measure="yes"/>`、duration は 1 小節分）を入れ、先頭の小節の `<attributes>` に
  `<measure-style><multiple-rest>4</multiple-rest></measure-style>` を付ける。段頭の小節番号はこれを数えた番号。
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
python3 $SKILL/scripts/scoretool.py split work/score.musicxml --measure 12 --at 2   # 小節線の見落とし: 2 拍目で分ける（全パート）
python3 $SKILL/scripts/scoretool.py split work/score.musicxml --measure 12 --before-note 7   # 7 番目の音の前で分ける
python3 $SKILL/scripts/scoretool.py merge work/score.musicxml --measure 12          # 余分な小節線: 12 と 13 をつなぐ（全パート）
```

- `split` / `merge` は中身（音符・強弱など）をそのまま移し、番号を振り直す。`--at` は小節頭からの 4 分音符数。
  分割位置をまたぐ音符があると警告して前の小節に残すので、その音符だけ直す。

- `replace`/`insert` は断片が整形式でなければ書き込まない。小節の追加・削除は全パートで揃える。
- MusicXML の書き方（divisions と duration、和音・backup、タイ・スラー、歌詞、強弱）は
  [references/musicxml.md](references/musicxml.md)。`divisions` は先頭小節の値を使い続ける。
- タイトル・作曲者・パート名（`<work-title>`, `<creator>`, `<part-name>`）も画像から直す。
  OMR は既定でタイトルに画像ファイル名を、パート名に `Part 1` を入れる。

## 5. 仕上げと報告

1. `check` が 0 件（弱起・最終小節の underfull は画像で確かめた上で許容）。小節数と拍子が画像と一致する。
2. 最後にもう一度全段を通して見て、直し漏れが無いか確かめる。
3. 成果物は `work/score.musicxml`（ユーザーの指定があればそこ）。ユーザーへの報告には次を含める:
   - 直した内容の要約（例: 「12 小節で音価、5 小節で音高、2 小節の欠落を補った、歌詞を全て入力」）
   - 不確かな箇所（小節番号・パート・理由）
   - 表現していないもの（例: 運指、ペダルの細部）
   - 下書きをどれだけ使えたか（例: 「187 小節中 150 小節は下書きのまま、30 小節は音価を部分修正、7 小節は書き直し」）と、
     `omr.log` の flagged の数・拍子の推定/補正の記録

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
| 拍子 | 段の全五線に拍子記号が見えないと読めない。読めない・中身と食い違うときは小節の中身から推定して直す（`omr.log` の pages に記録）。2/2 と 4/4、3/4 と 6/8 のように長さが同じものは区別できない |
| 小節番号・cue 音符 | 小節番号の数字や小さい cue 音符（他パートの手がかり音符）が、複数小節休符の数・連符・音符として紛れ込むことがある |
| 暗黙の 3 連符・声部 | 小節が溢れる場合の推定。多声部の声部割当て |
| 連桁の多い速い音型・密な和音 | 音価（連桁の本数）と和音構成音の欠落 |
| 小さい五線（オッシア）・1 線譜・打楽器譜・タブ譜 | 非対応。画像から書き起こす |
| 画質 | 低解像度・傾き・かすれ・手書き、スマートフォンで撮った・スキャンした JPEG の PDF では全体に精度が落ちる（細い符幹・小さい符頭を落とし、小節線を記号と取り違える）。この場合も構成（小節線・段）と音高の目安には使えることが多いので、まず `systems --printed` で構成を合わせる |
