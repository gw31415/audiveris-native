# Audiveris classifier — native Rust

Audiveris **5.11.0** (`9e1e55cd2746037d059345881c53e6a6754bffbd`) の分類器推論を実移植した独立crate。110個の抽出済み特徴から、149中間値・149クラスのsigmoid scoreを計算する。**glyph特徴抽出、画像前処理、OCR、SIG、リズム推定、MusicXML出力は未実装**。

Javaはテスト値を採取したoracleのみ。productionのlibrary/CLIはJVM・Java subprocess・JNI・BLAS・推論runtimeに依存しない。task-5のファイルは変更していない。

## 利用

```sh
cargo test --locked --offline
cargo build --locked --offline --release
./target/release/audiveris-classify \
  models/audiveris-5.11.0.json tests/fixtures/mean-features.json
```

`--offline` は依存crateがcache済みの環境用。このworkspaceで実行済み。CLI入力は110個の有限数からなるJSON配列。出力は `class_labels` と `scores`（双方149個、元モデル順）。不正入力はstderrへ理由を出しexit2、stdoutには成功JSONを出さない。`--version` / `--license` も利用可能。

library API:

```rust
use audiveris_classifier::Model;
let model = Model::from_json(&std::fs::read("models/audiveris-5.11.0.json")?)?;
let result = model.evaluate(&raw_features)?; // &[f64], length 110
let labels = model.output_labels();
// result.normalized[110], result.hidden[149], result.scores[149]
```

統合先のCargo.tomlからこのdirectoryをpath dependencyにできる。モデルは呼出側からbytesで渡すので、filesystemやCLIにlibraryは依存しない。

## 固定した数値契約

- 全演算f64。`(raw[i] - means[i]) / stds[i]`。
- hidden weightsは149×111、output weightsは149×150。各行のindex0がbias。
- 内積は入力indexの末尾から0へ計算し、最後にbiasを加える。明示的なFMA、BLAS、parallel reductionは使用しない。
- 両層とも `1 / (1 + exp(-sum))`。softmaxではなく、score総和が1になる保証はない。
- 入出力label順は公式モデルschemaと完全照合。元モデルのクラス順のまま返す。
- JSONの不正構文、非有限数、次元・行幅違い、ラベルの交換/重複、非正のstd、provenance不一致を拒否。inputの非有限値や有限範囲外の中間演算も明示エラー。
- sigmoidの有限引数からexpがoverflowした場合はJava式と同様に0へ飽和する。入力/内積が非有限のときまでJavaのNaN伝播を再現することはせず、エラーにする。
- JSONのsource_commit/hash文字列はschema照合であり、weight全体の暗号学的認証ではない。配布した固定モデルの実bytesは `ARTIFACT-MANIFEST.json` と `verify_artifacts.py` で照合する。

## 検証済み

**Java golden136ケース、55,488値をdebug/release双方で比較して合格**。136件=基礎・stress10件、basis110件、seed固定16件。入力は新規合成した数値で、学習譜面画像を使用していない。

初回比較前に固定した許容差:

```text
abs(actual - reference) <= 1e-12 + 1e-10 * abs(reference)
```

結果（debug/release同じ）:

|対象|最大絶対差|
|---|---:|
|正規化110値|0|
|中間149値|1.1102230246251565e-16|
|出力149値|2.220446049250313e-16|

top-5のindex/label順も全件一致。基準を通すための許容差変更はしていない。次元・順序・不正JSON・非有限値・std・provenance・overflow・CLIの検証を含め全6テスト合格。`cargo clippy --all-targets -- -D warnings` と `cargo fmt --check` も合格。

証拠: `verification/golden-debug.json`, `golden-release.json`, `dependency-tree.txt`, `binary-links.txt`, `toolchain.txt`。CLIは環境変数を最小化した `env -i PATH=/usr/bin:/bin` でも実行済み。共有ライブラリ一覧にJavaはない。

再生成:

```sh
AUDIVERIS_VALIDATION_REPORT=verification/golden-debug.json cargo test --locked --offline -- --nocapture
AUDIVERIS_VALIDATION_REPORT=verification/golden-release.json cargo test --locked --offline --release -- --nocapture
python3 verify_artifacts.py
```

## モデルとoracleの出典

- 原モデルZIP: `models/basic-classifier.zip`
  - SHA256 `327194c47116a8ebf5771d04cea08a4e563ec93d2955e1efd990a54a9245f29e`
- XMLを値・label順を保って変換したJSON: `models/audiveris-5.11.0.json`
  - SHA256 `4650334a8a1fe8d5f65e9bc53bee5f2df7fff85e90c53665d8f6f75805fc9a5f`
- 実Java出力: `tests/fixtures/java-5.11.0-golden.json`
  - SHA256 `432e7003d8c2b319575e79c023a6f378cd4f18f9092b5ce47fb8aa1d46dfe983`

元JavaのNeuralNetwork、PoorManAlgebra、BasicClassifierを `upstream/` に未改変保存。oracleは実5.11.0のNeuralNetwork/PoorManAlgebraをJava21.0.7で実行し、2つの独立JVMでbyte同一JSONを生成した。旧Audiveris本体をruntime classpathに含めていない。`upstream/oracle-provenance.json` は採取環境の記録（元workspaceの絶対pathを含む）。完全な採取harnessは隣接 `../oracle/` にあり、このcrateのビルド/テストには不要。

公式: https://github.com/Audiveris/audiveris/tree/5.11.0

## ライセンスと未確定事項

翻訳コードは **AGPL-3.0-or-later**。Audiveris copyright・原著者・pin・変更日をNOTICEと各sourceに保持し、LICENSE全文を同梱。公開/配布時はAGPLの対応ソース提供等の条件を満たす必要がある。この作業では外部公開していない。

元モデルZIPには独立のmodel licenseが入っていない。Audiveris同梱資産として原文AGPL・出典・元ZIP・変換履歴を保持するが、モデル専用の権利許諾と全学習データprovenanceが確定したとは主張しない。JSON化だけで別licenseへ変更していない。依存crateの個別license原文は `third-party/`、一覧は `THIRD_PARTY.md`。

この合格は**抽出済み110特徴からの分類推論**の移植確認であり、glyph認識精度やAudiveris全体の移植完了を意味しない。
