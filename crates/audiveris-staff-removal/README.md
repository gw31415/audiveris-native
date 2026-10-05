# audiveris-staff-removal

Audiveris 5.11.0 (`9e1e55cd2746037d059345881c53e6a6754bffbd`) の確定staff-line glyph消去と、残差画像からのGlyphFactory connected-component切出しをRustへ移植した独立library。**production依存はstdのみ**。Javaはfixture生成時の比較用だけで、製品runtimeには不要。

## leadへ渡す入力境界（固定）

```rust
pub fn erase_staff(
    page: BinaryPage<'_>,
    glyphs: &[GlyphPixels],
    limits: Limits,
) -> Result<EraseResult, Error>;

pub struct GlyphPixels {
    pub left: i32, pub top: i32,             // 整数のglyph画素原点
    pub width: usize, pub height: usize,   // glyph local box
    pub orientation: Orientation,         // Horizontal または Vertical
    pub runs: Vec<Run>,
}
pub struct Run { pub position: usize, pub start: usize, pub length: usize }
```

`runs` は**最終確定した画素membership**のforeground runs。local位置・開始は0基準、開始inclusive/終端exclusive、length>0。position/startの昇順で、同sequence内のrunは重複せず、隣接runはmerge済み。glyph間の重複画素は許可し、一度だけ除去する。

入力にStaff曲線・平均太さ・暫定section IDだけを渡してはいけない。leadの `completeLines`、線端調整、section追加後の最終membershipから `SectionCompound.toGlyph` 相当のrun unionを作る必要がある。曲線のY+0.5やXML点の丸めをglyph整数画素座標へ加えない。glyph boundsとStaffLineの幾何学的endpointは必ずしも同じでない。

このmoduleは暫定曲線からmaskを作らず、staff membershipを追加・補正しない。membershipで除去対象に指定された交差画素を自動復元することもない。本家と同じ白塗りを実行する。

`EraseResult.residual` はrow-major u8、黒0/白255。`removed_mask` は**実際に黒→白へ変化した画素が0、それ以外255**。元が白のglyph画素はremovedに数えない。負のglyph原点・画像端のはみ出しはGraphics2Dと同じくclip。空glyph一覧・runなしglyphはno-op。0サイズpage、非binary値、invalid runはエラー。

### leadが使えるexact path

workspace root: `/Users/ama/Documents/Codex/2026-10-02/task-6/audiveris-staff-removal`

- **最終30 glyph入力（正規化foreground runs）**: `/Users/ama/Documents/Codex/2026-10-02/task-6/audiveris-staff-removal/fixtures/official-staff-glyphs.json`
- **本家BINARY**: `/Users/ama/Documents/Codex/2026-10-02/task-6/audiveris-staff-removal/fixtures/binary.png`
- **本家NO_STAFF（期待残差）**: `/Users/ama/Documents/Codex/2026-10-02/task-6/audiveris-staff-removal/fixtures/official-no-staff.png`
- **期待actual removed mask**: `/Users/ama/Documents/Codex/2026-10-02/task-6/audiveris-staff-removal/fixtures/official-removed.png`
- **元の本家XML/RLE**: `/Users/ama/Documents/Codex/2026-10-02/task-6/full-java-oracle/runs/laura-original-rgba/GRID/omr-extracted/sheet#1/sheet#1.xml`

JSONには監査用の `staff_id`（文字列）、`line_index`（0基準）、`glyph_id`（文字列）を加えている。libraryはmembership描画にIDを使わない。JSONの `orientation` は `HORIZONTAL` / `VERTICAL`。metadataを除いて上記Rust型へ変換する例は `tests/official.rs`。このlibrary自体は無制限なJSON decoderやimage decoderを公開していない。

## 残差→connected glyph

```rust
pub fn extract_glyphs(
    page: BinaryPage<'_>,
    orientation: Orientation,
    origin: (i32, i32),
    limits: Limits,
) -> Result<Vec<GlyphPixels>, Error>;
```

`GlyphFactory.scanTable/createLut/buildAllGlyphs/buildGlyph` と同じ**4近傍**。対角だけの接触は別glyph。連続sequence間でrunがstrict overlapする場合だけ結合する。出力は最小祖先markの順（最初のscan出現順）で、面積順・座標sortを後から加えない。source orientationを維持し、glyph boxはtight、runsはglyph localへ移す。originをboxへ加算し、i32座標overflowはエラー。

merge pathを再帰ではなくunion-findの反復処理で解決し、戻らない前方runをskipする。Javaと同じ結果を保ちつつ、非接触checkerboardで不要な二重走査を避ける。単なる汎用8近傍CCLへの置換ではない。

## 本家比較・検証

- 実装前に本家30 glyph/residualを期待値にした試験を配置。Rust結果は **全画素一致**。除去54,466画素、残差135,311画素。
- 公式配布jarの実 `RunTableFactory` と `GlyphFactory` をreflection adapterから呼び、PNG入力から14 goldensを生成。Rust出力をJava入力に使っていない。
- 空、1画素、対角接触、分岐合流、出力順、横/縦orientation、および実残差を検証。**順序・run・box・原点・weightを全件exact一致**。
- 実残差は両orientationとも **604 connected glyphs**。これは604個の音符という意味ではない。
- mask試験は線とstem/符頭の交差、membershipによる保護、重複glyph、負origin・画像外clip、空、非binary、run不正、budget、coordinate overflowを検証。
- 256×256 checkerboardを4近傍で32,768 glyphに分離。再帰stackを使用しない。
- defaultは16M画素/辺65,536/100K glyph/1M runs/64M総run画素。A4 300 DPI相当2480×3508=8,699,840画素を受理し、16M超を拒否する試験あり。
- 除去の追加pixel buffersは `2×width×height` bytes。切出しはrun数に比例するrun/mark/親/出力storageで、画素数×ラベルの配列を作らない。全大規模Vec確保はfallible。capsは論理resource上限で、OS全process RSSのhard limitではない。呼出側が作る画像/JSON/glyph入力の確保は呼出側の責任。

```sh
cargo test --offline --locked
cargo test --release --offline --locked
```

coreの8 testsが成功（うちofficial比較testが14ケースを走査）。golden生成にはPillow、Java runtime、`oracle/GlyphOracle.java` のコンパイル済みclassが必要だが、Rustテスト自体にJavaは不要。

## features → classifierとの接続

`GlyphPixels` を既存 `audiveris-glyph-features::Geometry` にwidth/height/left/top/orientationをそのまま移し、**scaleで確定したinterlineを別途付与**する。`Run`はposition/start/lengthを同名型へ写し、`features::from_runs` の110 raw featuresを `audiveris_classifier::Model::evaluate` に渡す。centroidへ移したりboxをpaddingしたりしない。

動く接続例は `oracle/feature-bridge/src/main.rs`。既存の兄弟crateをpath依存で利用する検証用別projectで、production libraryへ依存を追加しない。

```sh
cargo run --offline --locked --manifest-path oracle/feature-bridge/Cargo.toml
```

実残差604 glyphすべてから110 features→149 finite sigmoid scoresまでnative Rustで通過した（interline11、rejected0）。結果は `oracle/feature-bridge-result.json`。connected componentは必ずしも1つの音楽記号ではない。今後の候補の分割/結合、文脈判定、SIGraph・rhythmが別途必要であり、この接続を完成した音符認識と呼ばない。

## 出典・未完了範囲

消去: `Picture.buildNoStaffBuffer` / `RunTable.render`。glyph生成: `GlyphFactory` / `RunTableFactory`。固定source・SHAは `oracle/` とmanifestを参照。AGPL-3.0-or-later、原著作権表示を保持。入力An LauraはCC0、来歴manifestを同梱。

未担当: final Staff membership生成、`StaffLineCleaner` の全workflow、HLAG再構築とsystem配属、音楽記号候補のsemantic grouping。task-5へ書込みは行っていない。
