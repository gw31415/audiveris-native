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

MCP からの実行は PATH が通らないことがあるので、`~/.cargo/bin/audiveris-native` と
`~/audiveris-native/skills/score-transcription/scripts/scoretool.py` を絶対パス（`$HOME/...`）で呼ぶ。
更新は同じ `cargo install` と `git -C ~/audiveris-native pull`。

## 流れ

以下の実行はすべて `start_execution(command=[...], io="pipes")`。終了は `wait_execution` で待ち、
標準出力を受け取る。

1. **セッション**: 作業ディレクトリ（例 `~/scores/<曲名>`）を `["mkdir","-p",DIR]` で作り、
   `open_session(cwd=DIR)` を開き直す。以後の実行はその中で行う。
   - `profile`: `sandbox`（書き込みはセッション内のみ・ネットワーク無し）が使えればそれが安全。
     ただしサーバーの bubblewrap 設定や mise などのシム（読み取り専用の環境で失敗する）によっては
     失敗するので、そのときは `host` を使う。

2. **画像を置く**（どれか）:
   - ユーザーがチャットに添付したファイル（claude.ai）: ツール一覧に `create_upload` があれば、
     `create_upload(session_id, path="score.pdf")` で URL を受け取り、会話側のサンドボックスで
     `curl -fsS -T /mnt/user-data/uploads/<ファイル> '<upload_url>'`。PDF もサイズを問わずそのまま送れる
     （必要なページだけ先にサンドボックスで切り出してもよい）。curl が通らなければ、claude.ai の
     コード実行でホストのドメインへの通信が許可されていない。ユーザーに許可を頼むか、下の方法へ。
   - ユーザーがチャットに添付したファイル（ChatGPT など）: `import_file`（`download_url` が許可されたオリジンのとき）。
   - URL で公開されている: `["curl","-fsSLo","page-1.png",URL]`（`host`）。
   - すでにサーバーにある: `cp` する。
   - 手元にしか無く、上のどれも使えない: 数十 KB までなら base64 にして断片ごとに書き（手順 5 の方法）、
     `["bash","-c","cat page-1.b64.* | base64 -d > page-1.png && rm page-1.b64.*"]`。
     それより大きいと書き写しの誤りでほぼ失敗するので、ユーザーに `scp` などでホストへ置いてもらう。
   PDF なら `["pdftoppm","-r","300","-png","score.pdf","page"]`。
   ページ順に並ぶ名前にする（10 ページ以上なら `page-01` のようにゼロ埋め）。

3. **OMR と材料**:
   ```json
   ["bash","-c","T=$HOME/audiveris-native/skills/score-transcription/scripts/scoretool.py; $HOME/.cargo/bin/audiveris-native --debug debug --report omr.log -o omr.musicxml page-*.png && cat omr.log && python3 $T crops debug crops page-*.png && python3 $T check omr.musicxml; python3 $T summary omr.musicxml"]
   ```
   出力が長いときは `summary --measures 1-16` のように分ける。`unknown option --report` で止まるなら CLI が古いので、
   サーバー側の準備の `cargo install` で更新する。

4. **画像を見る**: `get_image(path="crops/page-1.png")`（全体）、`crops/S01.png`、`crops/S01-1-2.png` …
   （1 MiB 以下。超えるときは `crops --split 1` で作り直す）。
   - `get_image` が失敗する場合（サーバーとクライアントの MCP プロトコルの不一致など）で、手元に
     ファイルを扱える環境（Claude Code など）があれば、`["base64","-w0","crops/S01.png"]` の出力を
     手元でデコードして画像として読む。それも無理なら、ユーザーに元画像を会話に直接添付してもらう。

5. **修正**: 最初に `["cp","omr.musicxml","score.musicxml"]`。直した `<measure>` を `fix.xml` に書いて
   `scoretool.py replace score.musicxml --part P1 --measure 5 fix.xml`。
   - 書き込みは `write_file(path="fix.xml", content=...)`。失敗する場合は引数で渡す:
     `["bash","-c","printf '%s' \"$1\" > fix.xml","_","<measure number=\"5\">…</measure>"]`
     （1 引数 64 KiB まで）。
   - `read_file` は 64 KiB までなので、楽譜全体を読まず `extract` / `summary` の出力で確かめる。

6. **成果物を渡す**: サーバー上のパスを伝える。手元に必要なら `read_file(score.musicxml)`（64 KiB 以下）か、
   `["split","-b","60000","score.musicxml","out."]` で分けて順に読み、連結する。
