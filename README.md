# audiveris-native

楽譜画像（PNG/JPEG/TIFF/BMP、PDF）を認識して MusicXML 4.0 を出力する Rust 製 CLI です。
[Audiveris](https://github.com/Audiveris/audiveris) 5.11 の認識パイプラインと同じ手順・同じ
アルゴリズム（同じ定数・テンプレート・分類器）を、必要な部分だけ素直に実装し直しています。
Java も Audiveris 本体も不要です。

```sh
cargo build --release
./target/release/audiveris-native page1.png page2.png -o score.musicxml
./target/release/audiveris-native -v --debug out/ page.png   # 工程ごとの時間、注釈画像と認識JSON
cargo test --release                                       # 実譜面1ページの end-to-end テスト
tools/evaluate.sh                                           # samples/ の正解との比較
```

複数画像は同じ曲の連続ページとして 1 つの楽譜にまとめます（拍子・調・音部記号はページを跨いで引き継ぎ）。

## 処理の流れ（Audiveris の各 step と対応）

| step | ファイル | 内容 |
|---|---|---|
| BINARY | `binary.rs` | AdaptiveFilter（37×37 窓、`0.7·mean + 0.9·std`） |
| SCALE | `scale.rs` | 縦ランのヒストグラム（線幅、black+white combo による五線間隔、連桁の太さ、符幹幅） |
| GRID | `staff.rs`, `grid.rs` | 細い横ランの「櫛」で五線を検出し、5本共通の滑らかな曲線で頑健にフィット。全高の縦ピークを小節線とし、接続で段（system）、ブレースでパートを構成 |
| HEADERS | `header.rs` | 段頭の音部記号・調号・拍子を分類器で認識（五線消去で割れた記号は隣接部分を結合して再評価） |
| BEAMS / LEDGERS | `beams.rs` | 連桁らしい厚みの画素から平行四辺形をフィット、加線 |
| HEADS | `heads.rs`, `template.rs` | Bravura の符頭を実測サイズでラスタライズし、fore/back/hole キーポイントによる Audiveris と同じテンプレート距離で、線・間・加線の各高さを走査 |
| STEMS / REDUCTION | `stems.rs` | 符頭の左右から符幹を探索。Audiveris の文脈グレード `(1+C)g/(1+Cg)`、尾の最小長 1.75、2度の反対側規則、臨時記号型の偽符頭排除 |
| SYMBOLS | `symbols.rs`, `classifier.rs`, `features.rs` | 認識済み要素を消した残差のグリフ（と近傍の組合せ）を Audiveris 5.11 の学習済みニューラル分類器で分類。旗・休符・臨時記号・付点・音部記号変更・強弱・連符数字など |
| CURVES | `curves.rs` | 残差グリフの包絡線から直線セグメントを取り、ヘアピン（WedgesBuilder）とボルタ（EndingsBuilder）を構成 |
| CHORDS / RHYTHMS | `score.rs` | 和音・休符・音価（旗/連桁/付点/連符）、音高（音部記号・調号・小節内臨時記号）、声部とタイムスロット、暗黙の3連符、隠れた空き五線を含むパート対応 |
| export | `musicxml.rs` | MusicXML 4.0 partwise（XSD 検証済み） |

## 精度

正解は MuseScore の原譜（OpenScore Lieder, CC0）です。`tools/compare.py` で、パート・五線ごとの
（音高または休符, 音価）列を整列して一致率（FLAT）を求めます。小節線の取り違えに左右されない指標です。
同じ画像を公式 Audiveris 5.11.0（Linux 版、既定設定）にも通して比較しました。

`samples/real`（実譜面ページ。An Laura / Der Winter は 180dpi、Der Brautschmuck は 180dpi の MuseScore 描画）:

| 曲 | 本実装 | Audiveris 5.11 |
|---|---|---|
| An Laura | 0.980 | 0.869 |
| Der Winter | 0.960 | 0.831 |
| Der Brautschmuck | 0.965 | 0.823 |

調整に一切使っていない検証用コーパス：シューベルト《冬の旅》全22曲（65ページ、MuseScore 4 で 300dpi 描画）:

| 曲 | Audiveris 5.11 | 本実装 |
|---|---|---|
| wr01 | 0.858 | 0.911 |
| wr02 | 0.983 | 0.970 |
| wr03 | 0.322 | 0.945 |
| wr04 | 0.973 | 0.941 |
| wr05 | 0.951 | 0.967 |
| wr06 | 0.971 | 0.962 |
| wr07 | 0.883 | 0.885 |
| wr08 | 0.986 | 0.961 |
| wr09 | 0.950 | 0.901 |
| wr10 | 0.719 | 0.822 |
| wr11 | 0.726 | 0.989 |
| wr12 | 0.875 | 0.887 |
| wr13 | 0.375 | 0.478 |
| wr14 | 0.911 | 0.865 |
| wr15 | 0.931 | 0.872 |
| wr16 | 0.950 | 0.940 |
| wr17 | 0.981 | 0.980 |
| wr18 | 0.808 | 0.945 |
| wr19 | 0.953 | 0.935 |
| wr20 | 0.988 | 0.951 |
| wr21 | 0.934 | 0.892 |
| wr22 | 0.977 | 0.983 |
| 平均 | 0.864 | 0.908 |

（wr18 は Audiveris が 6 ページ中 1 ページを処理できませんでした。）
処理時間は 300dpi の 1 ページで約 5 秒（1 スレッド）です。

コーパスの再現: OpenScore Lieder の `.mscx` を MuseScore で
`mscore -r 300 -o NAME.png NAME.mscx`（`NAME-1.png, NAME-2.png, …` が出力される）とし、
`tools/evaluate.sh コーパスのディレクトリ` を実行します。

## 開発方針

**Audiveris 本家と原則同じロジック**で実装し、本家と異なるロジック（改良）は、正解付きコーパスで
本家の方式より精度が上回ることを確認できた場合にのみ採用します。

優位性は《冬の旅》コーパスで、本家 Audiveris 5.11 の出力と次の観点別の指標を比べて確認しています
（単位ごとの差し替え実験ではなく、その単位が効く観点での比較）。

| 観点 | 指標 | 本実装 | 本家 |
|---|---|---|---|
| 音高側（五線・符頭・加線・音部記号・調号・臨時記号） | 音高のみの列の一致率（厳密 LCS） | 0.967 | 0.950 |
| 音価側（符幹・連桁・旗・付点・連符・声部） | 音価のみの列の一致率（厳密 LCS） | 0.937 | 0.873 |
| 小節線 | 小節数の誤差の合計（22 曲） | 8 | 15 |
| パート構成 | パート数の正解曲数 | 22/22 | 22/22 |
| 調号 | 調の列の正解曲数 | 22/22 | 21/22 |
| 拍子 | 拍子の列の正解曲数 | 22/22 | 21/22 |

## 機能チェックリスト

凡例: ✅ 本家と同じ ／ 🔧 改良ロジック（本家と異なる） ／ ⬜ 未実装

| 機能 | 実装単位 | 状態 | 備考（本家の該当クラス） |
|---|---|---|---|
| **画像入力・二値化** | 画像読込（PNG/JPEG/TIFF/BMP、グレー化） | ✅ | ImageLoading |
| | PDF入力 | ✅ | 本家と同じく 300dpi でラスタライズ（pdftoppm または Ghostscript） |
| | 二値化 | ✅ | AdaptiveFilter（37×37、0.7·mean+0.9·std） |
| **スケール** | 線幅・五線間隔 | ✅ | ScaleBuilder（black / combo ヒストグラム） |
| | 連桁の太さ | 🔧 | 長い横ランのみの縦ラン分布。本家の black ピーク方式では過小推定する例あり（優位性: 冬の旅で確認） |
| | 符幹の太さ | 🔧 | 横ランのピーク（本家 StemScaler）。優位性: 音価側の指標で確認 |
| | 第2の五線間隔（小さい五線） | ⬜ | ScaleBuilder interline2 |
| **五線** | 五線の検出 | 🔧 | 細い縦ランの「櫛」で検出（本家 LinesRetriever / ClustersRetriever）。優位性: 音高側の指標で確認 |
| | 線の形状モデル | 🔧 | 5本共通の二次曲線を頑健フィット（本家 NaturalSpline）。優位性: 音高側の指標で確認 |
| | 五線の消去 | 🔧 | 線上の細い縦ランを消去（本家は線 glyph の画素）。優位性: 音高側の指標で確認 |
| | 1線譜・打楽器譜・タブ譜 | ⬜ | OneLineStaff, Tablature |
| **小節線・段・パート** | 小節線の検出 | 🔧 | 全高の縦ピーク＋端の検査（本家 BarsRetriever）。優位性: 小節数の誤差 8（本家 15） |
| | 段の構成（コネクタ） | ✅ | 小節線が五線間をつなぐ段 |
| | ブレース／ブラケットとパート | 🔧 | 左端の大きな成分の形で判定（本家 BracesRetriever / PartsBuilder）。パート数は本家と同等（22/22） |
| | 小節線の種類（複縦線・終止線） | 🔧 | 太さで判定 |
| **音部記号** | 段頭の音部記号 | 🔧 | 分類器は本家と同じ、候補の組み立ては独自（本家 ClefBuilder）。優位性: 音高側の指標で確認 |
| | 途中の音部記号変更 | ✅ | SYMBOLS の分類器（G/F/C、小型を含む） |
| | 8va/8vb 付き音部記号 | ✅ | 分類器の形状 |
| **調号** | 段頭の調号 | 🔧 | 分類器による♯/♭の列（本家 KeyBuilder の射影ピーク方式）。優位性: 調の列 22/22（本家 21/22） |
| | 途中の調号変更・取消（ナチュラル） | 🔧 | 本家 KeyBuilder（複縦線の後）。段頭と同じ分類器方式に、音部記号が決める♯/♭/♮の段の順序と後続の空白の検査を追加（冬の旅: FLAT 0.904→0.906） |
| **拍子** | 段頭の拍子 | 🔧 | 分類器（全体形／分子・分母）＋分母の補完（本家 TimeBuilder の射影方式とは別）。段の全五線に拍子があることを要求（本家 TimeColumn と同じ）。冬の旅: 拍子の列 22/22 曲正解（本家 21/22） |
| | 途中の拍子変更 | 🔧 | SYMBOLS の拍子形状。段頭と同じく全五線での一致を要求 |
| **符頭** | テンプレート生成（Bravura、fore/back/hole） | ✅ | TemplateFactory |
| | テンプレート距離 | ✅ | Template.evaluate（重み 6/1/4） |
| | 符頭サイズの推定 | 🔧 | 黒符頭幅の実測（本家 BlackHeadSizer）。優位性: 音高側の指標で確認 |
| | 範囲走査（線・間・加線） | ✅ | NoteHeadsBuilder.lookupRange |
| | 符幹シードからの走査 | 🔧 | 本家 NoteHeadsBuilder.lookupSeeds。範囲走査が全ての横位置を評価するため、その候補に含まれる |
| | 重複・重なりの整理 | ✅ | aggregateMatches, HeadInter.overlaps |
| | 黒→白符頭の判定（hole） | ✅ | evalBlackAsVoid |
| | 小さい符頭（cue） | 🔧 | 小テンプレート（0.67）で範囲走査し、符幹付きのものを装飾音符に使用（本家は処理スイッチで既定オフ） |
| | 特殊符頭（×・菱形など） | ⬜ | ShapeSet.Heads |
| **符幹** | 符幹の探索 | 🔧 | 符頭の左右で縦の細い線を探索（本家 StemsRetriever / HeadLinker）。優位性: 音価側の指標で確認 |
| | 文脈グレードによる取捨 | ✅ | GradeUtil.contextual、HeadStemRelation 係数 4 |
| | 尾の最小長 | ✅ | StemInter.minTailLength 1.75 |
| | 2度の反対側規則・臨時記号型の偽符頭排除 | 🔧 | 本家 SIG の排他関係の簡略版 |
| **連桁** | 連桁の検出 | 🔧 | 厚みのそろった画素の平行四辺形フィット（本家 BeamsBuilder のスポット方式）。優位性: 音価側の指標で確認 |
| | 符幹との接続・レベル | 🔧 | 本家 BeamLinker |
| | 部分連桁（フック） | 🔧 | |
| | 小さい連桁（cue） | 🔧 | 本家 CueBeamsStep。装飾音符の符幹先端で旗・小連桁の本数を数えて音価に（冬の旅: 装飾音符の音価 112/209 正解、本家 0） |
| **旗** | 旗の認識 | 🔧 | 分類器は本家と同じ。符幹先端での限定分類を追加 |
| **加線** | 加線の検出 | 🔧 | 本家 LedgersBuilder とは別実装。優位性: 音高側の指標で確認 |
| **休符** | 休符の分類 | ✅ | 分類器 |
| | 全休符・2分休符の判別 | ✅ | 線からぶら下がる／線に乗る位置 |
| | 全小節休符 | ✅ | 小節中に単独の全休符 |
| | 複数小節休符 | ✅ | MultipleRestsBuilder（中央線上の 4 間以上の太線・両端のセリフ・上の数字）。冬の旅には無く合成画像で確認 |
| **付点** | 付点の認識と音符への割当 | 🔧 | 本家 DotFactory / AugmentationDotInter |
| | 和音の付点の一貫性 | 🔧 | 本家は各音の付点数の平均を丸める（countDots） |
| **記号の候補生成** | 部品グラフ（0.8間隔）と連結部分集合（最大7部品）の評価 | ✅ | Glyphs.buildLinks、GlyphCluster.decompose |
| | 候補の採否 | 🔧 | 本家は SIG の文脈グレードで解決。本実装は形状検査＋グレード順の貪欲選択（強弱記号・フェルマータは複数部品の記号を優先） |
| **臨時記号** | 分類 | ✅ | 分類器 |
| | 音高位置（♭の補正） | ✅ | AlterInter.computePitch |
| | 符頭への割当 | 🔧 | 本家 AlterHeadRelation |
| | 小節内での持続・調号との関係 | ✅ | |
| **和音** | 同じ符幹の符頭をまとめる | ✅ | ChordsBuilder |
| **装飾音符** | 装飾音符の記号（GRACE_NOTE 形状）と主音符への接続 | ✅ | GraceChordInter、SmallChordInter.lookupLink |
| | 小さい符頭＋符幹による装飾音符（16分・連桁付きの装飾音群） | 🔧 | 本家は全体形状の分類のみ（冬の旅: 再現率 0.67／適合率 0.86、本家 0／0）。接続は最寄りの符頭まで 3 間・右隣の装飾音経由も可 |
| **連符** | 明示された3連符・6連符 | 🔧 | 数字の近くの連桁群／3つの和音 |
| | 暗黙の3連符 | 🔧 | 小節が溢れる声部で、ちょうど収まる場合のみ（優位性: 冬の旅で確認） |
| **リズム** | 声部の割当 | 🔧 | 本家 Voices / SlotsRetriever とは別実装。優位性: 音価側の指標で確認 |
| | タイムスロット | 🔧 | |
| | 弱起・反復で分割された小節・空の小節 | ✅ | MeasureFixer（ページ先頭の短い小節＝弱起、後方反復をはさむ短い2小節＝分割、段頭の空小節は次と結合）。冬の旅: 7/10（本家 8/10）、誤検出 0 |
| **強弱記号** | p/f/mf/sfz など | 🔧 | 分類器・候補生成は本家と同じ。採用閾値 0.6 と複合記号（fp, mf…）の優先は改良（冬の旅: F1 0.82、本家 0.67） |
| **アーティキュレーション** | スタッカート・アクセント・テヌート・マルカートなど | ✅ | ArticulationInter.lookupLink、ChordArticulationRelation、DotFactory（スタッカート） |
| **装飾・その他記号** | フェルマータ | ✅ | FermataInter.lookupChordLink（冬の旅: F1 0.57、本家 0.56） |
| | トリル・ターン・モルデント | ✅ | OrnamentInter.lookupLink、ChordOrnamentRelation（冬の旅: 再現率 0.53／適合率 0.21、本家 0.53／0.12） |
| | アルペジオ | ✅ | ArpeggiatoInter.lookupLink、本家の最小グレード 0.15（冬の旅: 再現率 0.25／適合率 0.67、本家 0.17／1.0） |
| | ブレス・カエスーラ | ✅ | AbstractPauseInter.lookupLink（冬の旅には正解データなし） |
| | トレモロ（単音） | ✅ | TremoloInter.searchLinks（符幹／全音符）。2音間トレモロは本家も未対応 |
| **ヘアピン** | 直線セグメントの抽出 | 🔧 | 本家は骨格化した弧（SegmentsBuilder）。本実装は残差グリフの上下包絡線を直線当てはめ（傾き 20°・平均距離 0.3・run 比 0.75 は本家と同じ） |
| | 2本のセグメントの対応付けとグレード | ✅ | WedgesBuilder（冬の旅: 再現率 0.95／適合率 1.0、本家 0.90／0.99） |
| **オッターヴァ・ペダル** | オッターヴァ（8va/15ma/22ma） | 🔧 | OctaveShiftInter（記号の段の和音を対象、音高に反映）。本家は既定長で破線を確かめないが、OCR が無いため破線の追跡を必須に（冬の旅: 唯一の 8va を検出、誤検出 0。なお MuseScore の正解は記譜音高で持つため FLAT は wr20 で 5 音減） |
| | ペダル記号 | 🔧 | PedalInter（上の和音に接続）。OCR が無いため複数五線パートの最下段の下に限定（冬の旅には無く誤検出 0、合成画像で確認） |
| **反復構造** | 反復記号（縦線の点） | ✅ | DotFactory / RepeatDotInter（点の対・小節線から 1.5 間・段の過半数）。冬の旅: 3/3（本家 3/3） |
| | セーニョ・コーダ・D.S.・D.C. | ✅ | MarkerInter（重なる小節線に接続）。冬の旅には無く合成画像で確認（誤検出 0） |
| | ボルタ（括弧） | 🔧 | EndingsBuilder の長さ・傾き・五線からの距離・小節幅比。左の脚を必須に（冬の旅で誤検出 0、本家 1）。番号は OCR なしのため連番 |
| **タイ・スラー** | スラー／タイの検出と音符への接続 | ⬜ | SlursBuilder, SlurInter |
| **テキスト** | 歌詞 | ⬜ | LyricItemInter（OCR） |
| | 発想記号・テンポ・タイトル | ⬜ | WordInter, SentenceInter（OCR） |
| **ページ・パート** | 複数ページの結合・状態の引継ぎ | 🔧 | 本家 Book / PageRhythm |
| | 段ごとのパート対応（隠れた五線） | 🔧 | 本家 PartsBuilder / ScoreReduction。優位性: 冬の旅で確認 |
| **出力** | MusicXML 4.0 | ✅ | 本家 PartwiseBuilder（独自 writer、XSD 検証） |

## ライセンス・由来

AGPL-3.0-or-later（Audiveris に準拠）。`assets/classifier.json` は Audiveris 5.11.0 の学習済みモデル、
`assets/Bravura.otf` は SIL Open Font License（`assets/BRAVURA-OFL.txt`）。
`src/features.rs` と `src/labels.rs` は以前の検証済み移植（Audiveris の MixGlyphDescriptor / モデル定義）を
そのまま利用しています。`samples/real/*.mscx` は OpenScore Lieder（CC0）。

## 以前の実装について

このリポジトリには以前、Audiveris の内部 API を細粒度に一対一移植する方針の実装（約12.5万行と大量の検証証跡）がありました。
実譜面から MusicXML を出力できないまま停滞していたため、本実装に置き換えました。旧実装は git 履歴
（コミット `6a42c37` 以前）にそのまま残っています。
