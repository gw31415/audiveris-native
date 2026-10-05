# 通常Git workspaceへの移行

private `gw31415/audiveris-native` はルートにRust workspaceを置き、clone後に直接ビルドする方式へ移行します。符号化bundle/patchの連鎖を今後の通常保存に使いません。

```sh
git clone https://github.com/gw31415/audiveris-native.git
cd audiveris-native
cargo build --locked --bin audiveris-musicxml
cargo fmt --all -- --check
cargo clippy --workspace --locked --all-targets -- -D warnings
cargo test --locked --test musicxml_limited
./target/debug/audiveris-musicxml fixtures/official-image-musicxml-header/g-major-header-quarter.png g-major.musicxml g-major.json
```

Rust2024対応toolchainが必要です。認識の実行にはJavaを使いません。低容量Linuxでは `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` と `-j 1` を使えます。Cloud検証は既存Cargo cacheで `--offline` を使用しますが、通常cloneの構築に復元scriptは不要です。

移行はmainへの追加commitです。旧mainの全履歴とarchive/checkpointsは保持し、内側のソース実装履歴を同じrepoの`source-history`参照に保存します。mainルートは新たな開発の正本です。過去のbundle等は過去時点の復元資料として残ります。force push/rewriteはしません。

実行物は同じprivate repoのReleasesを配布先とします。ただし今回環境の既存Git接続は成功してもGitHub API（api.github.com）はForbiddenでした。Releases公開/asset uploadは未実行です。新認証・ネットワーク設定変更・別資格情報での回避は行いません。旧保存実行物は既存checkpointsに保持し、現在ソースからCLIを再ビルドできます。

全Audiveris機能の同等性は未完です。機能と例は[MUSICXML-LIMITED.md](MUSICXML-LIMITED.md)、原版全機能の移植台帳は[PORT-STATUS.md](PORT-STATUS.md)を参照してください。HEADERの固定調号伝播は原版成功1画像で追加検証した単位であり、他clef/time、一般調号/臨時記号、複声部、実スキャンなどの全面対応を意味しません。
