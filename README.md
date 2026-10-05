通常Git workspaceへの移行手順と履歴の扱いは[こちら](docs/GIT-WORKSPACE.md)。ルートから直接Cargo buildできます。

## 現在の目標と移植台帳（2026-10-05）

UI表示を除く原Audiveris5.11の認識・解析・補正・保存・MusicXML出力機能の同等性が最終目標です。原版機能を省略する予定はありません。限定画像CLIは小段階の成果です。[実装済み/部分/未対応/GUIのみ除外と接続順](docs/PORT-STATUS.md)、[現段階CLI手順](docs/MUSICXML-LIMITED.md)。以下の個別記録は当時の境界・ソースSHAとして保持します。

直近の追加は、認識済み固定3/4 TimePairから小節長・6音（F♯を含む）をMusicXMLへ伝える縦経路です。[入力例と実行手順](docs/MUSICXML-LIMITED.md#recognized-34-header--musicxml)。続けて固定COMMON_TIMEの実glyph・HEADER競合除外・`symbol="common"`を8音の原版成功画像で接続しました。4/4の既存入力を保持し、他拍子・他clef等は台帳に残して段階的に接続します。

# Audiveris native port — experimental page glyph pipeline

[Linux上の独立検証・音部記号候補の登録](docs/CLEF-REGISTRATION.md)。Javaを使わないnative実行で、候補の理論外形・重心補正・競合関係・header stopまで追加しました。調号・拍子後の最終clef選択とheader固定まで接続済みです。対応範囲は [HEADERS検証境界](docs/HEADERS.md) を参照してください。

[調号探索の範囲・画素投影・HiLoピーク](docs/KEY-PREPARATION.md) もnative化し、元画像10件と閾値・境界の独立oracleで比較しています。調号の分類・選択は後段のkey-state/key-columnへ接続済みです。

[調号要素の抽出・分類と空スライスの再試行](docs/KEY-COMPONENTS.md) を追加しました。分断された記号や重複候補も公式実装と比較済みです。[調号の状態処理](docs/KEY-STATE.md) と五線間の調整・複製・最終化まで接続し、51画像で公式KeyColumn結果と比較しています。MusicXMLまでの完了を意味しません。

[拍子候補のnative抽出](docs/TIME-CANDIDATES.md) は調号後の範囲、画素投影、記号分解、分類、理論外形を71画像で比較しています。[分子・分母の組合せとsystem全体の拍子決定](docs/TIME-COLUMN.md)、最終clef選択・bar除去・固定まで接続し、71画像で元実装のlive HEADERS状態と比較しています。音符・拍・MusicXMLは引き続き未完了です。

[STEM_SEEDS](docs/STEM-SEEDS.md) もnative接続しました。11画像でstem幅・候補フィラメント・採用glyph・画素サンプリングを公式結果と比較しています。これはstemのseed候補であり、音符・拍・MusicXMLの完成ではありません。

[BEAMSの画素処理・spot抽出](docs/BEAM-PREPARATION.md) を追加しました。`beam-spots INPUT SPOTS.pgm REPORT.json` はnativeで候補画像と未割当glyphを出力し、13画像・483候補を公式結果と比較しています。BeamInter・音符認識・MusicXMLは未完了です。

[BEAMS前処理のサイズ推定・system割当](docs/BEAM-PROLOG.md) まで追加しました。音符頭のサイズ・フォント推定、glyphの同一性とグループの保持、段ごとの候補割当を13画像で公式prologと比較しています。連桁そのものの確定・音符/拍・MusicXMLは引き続き未完了です。

[BEAMSの初期候補生成](docs/BEAM-CANDIDATES.md)を追加しました。`beam-spots` は beam/hook の幾何・画素採点・候補 glyph と排他関係を JSON に出力します。公式の初期候補生成を13画像で比較しています。連桁の延長・後段 hook・グループ・休符は未移植なので、BEAMS全体・音符/拍・PDF・MusicXMLは未完了です。


[線補完・全画像glyph抽出と検証境界](docs/LINE-COMPLETION.md)。`page-glyphs` はnativeスタッフ線画素生成→除去→連結成分→特徴量→任意の分類スコアまで接続します。

`page-glyphs` のJSONには [GRID IR](docs/GRID-FINAL-IR.md) も含まれます。永続StaffLine・画素で重複排除したglyph参照・再構築HLAG・system所属・通常小節線のgraphを出力します。[brace・part/pageの追加検証](docs/BRACES-PARTS.md) でAnLauraの全最終GRID graph内容と2-part構成が一致。[GRID意味処理・自動マスク](docs/GRID-SEMANTICS.md) でbracket/C-clefと右端補正の順序を追加し、13画像を公式結果と比較しました。一般的なGRID完了・音符認識は引き続き未完了です。

[固定テンプレート・全画像スコア比較](docs/TEMPLATE-PROBES.md) を追加しました。`template-probes` は画像を二値化し、明示された位置のテンプレートスコアを計算します。候補位置の自動探索・音符認識は未実装です。

`staff-mask INPUT MASK.pgm REPORT.json` は五線glyphと交差部を含む曲線からマスクを自動生成します。HEADS用の加線・stem seedマスクと音符候補探索は未完了です。

Rust CLIでAudiverisのロジック移植を開始したものです。**全体移植・音符認識は未完了**です。Java CLIのwrapperではありません。

```sh
cargo build --locked -j 2
cargo run --locked -- binarize fixtures/original-exercise.png /tmp/exercise-binary.pgm /tmp/exercise-runs.json
cargo run --locked -- scale fixtures/scale/clean-16.pgm /tmp/scale-binary.pgm /tmp/scale-report.json
cargo test --workspace --locked -j 2
cargo clippy --workspace --locked --all-targets -j 2 -- -D warnings
```

出力先が既に存在する場合は拒否します。入力は8-bit grayscale PNG（alphaなし・非animation）、またはPGM P2/P5（maxval 255）、最大400万画素・32MB。P5のheader/raster境界は1バイトの空白です。カラーPNG/JPEG/TIFF/PDFはこの段階では未対応です。

実移植した範囲:

- `AdaptiveFilter` + `VerticalFilter`: 37×37近傍の積分画像、平均・母標準偏差、`0.7 × mean + 0.9 × stddev`、`pixel <= threshold`。
- `RunsRetriever.processPosition`: 全画像・全run採用時の水平/垂直run抽出。
- native PNG/PGM入力 → 二値PGM + version付きJSON（寸法、前景数、3点の中間値、両方向の `[start,length]` 列）。JSONは独自の前処理IRで、Audiveris `.omr` 互換ではありません。

追加した `scale` commandは、二値化に続いて `ScaleBuilder` / `HiLoPeakFinder` を実行します。線幅・五線間隔（主/小）・連桁幅（実測/推定）と、black/combo histogram・選択peakをJSONの `scale` に追加します。成功exit 0、scale失敗exit 2（診断JSON・二値画像は保持）、入力/IOエラーexit 1です。5種類の失敗コードを区別します。上流のdefault設定・自動推定経路のみで、指定scaleの上書き機能は未対応です。

[省メモリ化とscale検証](docs/SCALE-MILESTONE.md) / [18ケースJava比較](evidence/scale-parity.json)

`grid-prep INPUT BINARY.pgm REPORT.json` はscaleに続いて水平/垂直runの振分けとsection生成を実行します。これはGRIDの入力部品までを出力する段階コマンドです。`grid_preparation.staff_detection` は `not_implemented` と明示します。[GRID前段の境界とJava比較](docs/GRID-PREP.md)。

独立検証済みclassifierを履歴付きで `crates/audiveris-classifier` に統合しました。`classify-features` は **110特徴のJSON配列**を受け取り、149クラスのlabel/scoreをstdoutへ出します。`classify-glyph` は孤立した二値記号画像から110特徴量を抽出し、同じclassifierへ直接渡します。両経路ともJavaや別プロセスは呼びません。

```sh
cargo run --locked -- classify-features \
  crates/audiveris-classifier/models/audiveris-5.11.0.json \
  crates/audiveris-classifier/tests/fixtures/mean-features.json
```

```sh
cargo run --locked -- classify-glyph \
  crates/audiveris-classifier/models/audiveris-5.11.0.json \
  fixtures/glyph/ellipse.pgm 8 horizontal 0 0
```

`classify-glyph MODEL.json BINARY_IMAGE INTERLINE [horizontal|vertical [LEFT TOP]]` は0/255の二値画像と明示した五線間隔を必要とします。画像のpaddingを保持し、run方向と原点も指定できます。stdoutは110 raw features・正規化値・149 hidden values・149 label/scoreのJSONです。五線除去・ページからの記号分離・楽譜上の意味付けは含みません。スコアはsigmoid出力で、合計1の確率ではありません。

[孤立記号96ケース・49,728値のJava比較](evidence/glyph-integration.json) / [特徴量抽出の数値契約とOFL fixture](crates/audiveris-glyph-features/README.md)。不正な二値画像・空glyph・interline=0等はexit1です。

[root CLI統合検証](evidence/classifier-integration.json) / [classifierの数値契約・モデルの権利未確定事項](crates/audiveris-classifier/README.md)。root CLIで不正特徴/モデルはexit1・成功JSONなしです。独立crate側CLIの不正入力exit2とは異なります。

`filament-seeds INPUT BINARY.pgm REPORT.json` は公式の初期filament選別と、矩形seedに限定したcomb列検出を追加します。結合・曲線補間・Staff生成は未実装で、5本組は診断候補です。[実装境界・Java比較・画像](docs/FILAMENT-SEEDS.md)。

`filaments INPUT BINARY.pgm REPORT.json` は矩形限定を外し、公式の曲線補間・結合/拡張・曲率/傾斜フィルタ・comb検出まで実行します。正式なStaff生成は未実装です。[今回の移植範囲とJava比較](docs/CURVED-FILAMENTS.md)。

`staff-candidates INPUT BINARY.pgm REPORT.json` は通常の5線・単一interlineのクラスタ形成と初期Staff幾何情報まで実行します。CC0実譜面で6 Staff、合成長尺譜面で1/2 StaffをJava由来oracleと段単位比較しました。後続GRID・systems・記号切出しは未実装です。[範囲・検証・制約](docs/STAFF-CLUSTERS.md)。

`grid-candidates INPUT BINARY.pgm REPORT.json` は小節線候補・縦filament・初期systemまで追加します。本家の元画像からの独立実行と、実譜面2 systems・合成1 systemを比較しました。no-system/未対応分岐はexit 2。**後続の線端・membership補正を含む全GRIDは未完了**です。[独立比較と残作業](docs/GRID-PROGRESS.md)。

現在の優先目標は画像 → 認識した音楽的内容 → MusicXML CLIです。[限定経路と検証範囲](docs/MUSICXML-LIMITED.md)の `audiveris-musicxml` は入力画像から認識した音符・四分休符・音価をnativeで出力します。原版XMLの流用はありません。独立した合成3譜面（2/4小節・改行あり）の内容一致と複数画像CLIを確認しています。任意の実譜面への適用と全工程一致は未完了です。上の各段階の説明は当時の移植境界です。

[移植境界・依存・次段階](docs/PORTING.md) / [検証結果](evidence/java-parity.json) / [帰属・ライセンス](NOTICE) / [fixture由来](fixtures/README.md)

Java oracle再実行（開発時の独立比較専用。製品実行には不要）:

```sh
python3 scripts/linux_oracle.py --bootstrap --probe clef-candidates
python3 scripts/linux_oracle.py --probe clef-registration
python3 scripts/linux_oracle.py --probe key-preparation
python3 scripts/linux_oracle.py --probe key-peaks
python3 scripts/linux_oracle.py --probe key-components
python3 scripts/linux_oracle.py --probe key-fallback
```

ハッシュ固定の公式Linux配布物とJDKを作業領域へ用意します。Macやシステム全体へのインストールは不要です。各回の結果は新しい証跡ディレクトリへ保存し、既存fixtureは上書きしません。[依存・再現方法](docs/LINUX-ORACLE.md)。

`upstream/`, `tools/`, `target/`, `evidence/generated/` は取得/生成物でGit対象外です。GUI操作・外部公開は行っていません。


限定MusicXML経路に、原版5.11が画像から正しくexportしたC4加線付き1譜面（8四分音符・2小節）を追加確認しました。具体的な加線の保持・消去と2部品評価を接続しています。中央四分休符の既知入力は原版も誤認するため拒否を維持します。他の加線音高・実譜面一般への対応を意味しません。詳細と実行例は[限定MusicXML手順](docs/MUSICXML-LIMITED.md)を参照してください。


限定経路に、既存の四分休符画像へ1pxの分断を入れた1入力を追加比較しました。原版5.11通常画像処理の8イベントとnativeが一致し、2部品のcanonical glyph・RestInter・chord登録からMusicXMLまで接続しています。限定Rest/TimeWholeの重なり・SIG排他整理も原版の個別条件と比較しています。真正な付点、support/frozen/異種間の競合、3部品以上は未対応です。

固定HEADER調号の実glyph/SIG→音高→MusicXMLをG major1画像で追加確認しました。F♯を含む8音が原版exportと一致。105調号kernelの一致と全ページ対応は別範囲です。[入力例・残タスク](docs/MUSICXML-LIMITED.md)。

固定HEADER 4/4 TimePairも1制御入力で画像→数字glyph/所有SIG→小節拍数→MusicXMLへ接続しました。原版の同一画像8音/F♯と一致。一般拍子/音部記号/変更は未対応です。[入力と比較範囲](docs/MUSICXML-LIMITED.md)。
