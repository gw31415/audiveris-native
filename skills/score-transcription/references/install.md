# audiveris-native の導入（ローカル）

Rust（rustup）があれば:

```sh
cargo install --locked --git https://github.com/gw31415/audiveris-native
audiveris-native --help
```

- 学習済みモデルとフォントはバイナリに埋め込まれるので、追加のファイルは不要。
- 歌詞・タイトル・発想記号の読み取りには Tesseract（任意）:
  `apt install tesseract-ocr tesseract-ocr-deu`、`brew install tesseract tesseract-lang` など。
  無ければそれらの無い MusicXML になる（このスキルでは画像から補う）。
- `scoretool.py crops` には Pillow（`pip install pillow`）か ImageMagick（`convert`/`magick`）。
- PDF を扱うなら poppler（`pdftoppm`）。

インストールせずに使うなら、リポジトリを clone して `cargo build --release` し、
`target/release/audiveris-native` を使う。
