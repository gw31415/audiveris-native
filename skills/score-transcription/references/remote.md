# リモートの CLI を使う（dev-session-mcp など）

CLI を手元に入れず、VPS 等に置いた `audiveris-native` をシェル系 MCP 経由で使う手順。
ツール名は dev-session-mcp のもの（`open_session` / `start_execution` / `wait_execution` /
`read_execution` / `get_image` / `write_file` / `read_file` / `import_file`）。他の MCP でも
「コマンド実行」「画像の取得」「ファイルの書き込み」に読み替えれば同じ流れになる。

## サーバー側の準備（1 回だけ）

```sh
cargo install --locked --git https://github.com/gw31415/audiveris-native        # CLI
git clone --depth 1 https://github.com/gw31415/audiveris-native ~/audiveris-native  # scoretool.py 用
# 任意: 歌詞 OCR と切り出し
sudo apt install tesseract-ocr tesseract-ocr-deu python3-pil   # または imagemagick
```

以下 `$SKILL` は `~/audiveris-native/skills/score-transcription`、
`$BIN` は `~/.cargo/bin/audiveris-native`（MCP のコマンドは PATH が通らないことがあるので絶対パスで）。
更新は同じ `cargo install` と `git -C ~/audiveris-native pull`。

## 流れ

1. **セッション**: `open_session(cwd=<作業ディレクトリ>)`。作業用ディレクトリ（例 `~/scores/<曲名>`）は
   `start_execution(["mkdir","-p",...], profile="host")` で作る。セッションの cwd はその中にすると、
   以後の実行を `profile="sandbox"`（書き込みはセッション内のみ・ネットワーク無し）で安全に行える。

2. **画像を置く**（どれか）:
   - ユーザーがチャットに添付したファイル: `import_file`（`download_url` が許可されたオリジンのとき）。
   - URL で公開されている: `start_execution(["curl","-fsSLo","page-1.png",URL], profile="host")`。
   - すでにサーバーにある: そのパスを使う。
   - 手元にしか無い: base64 にして 60KB 以下の断片ごとに `write_file`（`page-1.b64.000` …）、
     `bash -c 'cat page-1.b64.* | base64 -d > page-1.png && rm page-1.b64.*'`。
   PDF なら `pdftoppm -r 300 -png score.pdf page`。

3. **OMR と材料**（`profile="sandbox"`、`io="pipes"`）:
   ```
   ["bash","-c","$BIN --debug debug -o omr.musicxml page-*.png && python3 $SKILL/scripts/scoretool.py crops debug crops page-*.png && python3 $SKILL/scripts/scoretool.py check omr.musicxml; python3 $SKILL/scripts/scoretool.py summary omr.musicxml"]
   ```
   `page-*.png` はページ順に並ぶ名前にする（`page-10` があるなら `page-01` のようにゼロ埋め）。
   `wait_execution` で終了まで待ち、標準出力（要約と check の結果）を受け取る。出力が長いときは
   `summary --measures 1-16` のように分けて実行する。

4. **画像を見る**: `get_image(path="crops/page-1.png")`（全体）、`crops/S01.png`、`crops/S01-1-2.png` …
   （1 MiB 以下。超えるときは `crops --split 1` やより小さい `--pad` で作り直す）。
   注釈画像 `debug/<名前>.png` は大きいので、必要なら
   `convert debug/x.png -resize 50% dbg-small.png` で縮めてから見る。

5. **修正**: 直した `<measure>` を `write_file(path="fix.xml")` で書き、
   `scoretool.py replace score.musicxml --part P1 --measure 5 fix.xml` を実行（最初に
   `cp omr.musicxml score.musicxml`）。`read_file` は 64 KiB までなので、楽譜全体を読むのではなく
   `extract` / `summary` の出力で確認する。

6. **成果物を渡す**: パスを伝える。手元に必要なら、`read_file(score.musicxml)`（64 KiB 以下のとき）か、
   `split -b 60000 score.musicxml part.` で分けて順に `read_file` して連結する。
