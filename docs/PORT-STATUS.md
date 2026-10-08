# 画像入力から原Audiveris同等品質のMusicXMLへ：現在位置と接続順

更新日: 2026-10-08。対象原版5.11.0、ソース `9e1e55cd2746037d059345881c53e6a6754bffbd`。

ユーザーが終了条件を変更しました。現在のゴールは、少なくともPNG画像入力から、
代表的な実譜面を原版Audiverisと同等品質で認識し、native Rust CLIでMusicXMLへ出力することです。
PDF/多形式変換、一般編集CLI/UI/undo、project保存互換、認識に不要な原版全API再現は対象外です。
認識に必要な実所有・関係・callback・rhythm処理は保持し、画像→認識→出力の未接続箇所を優先します。
旧「非UI全APIを省略なく移植」goalと、そのための工程はこの変更で終了条件から外れました。
下の機能別台帳・古い工程説明・checkpointは当時の証拠として保持し、現在の残作業一覧とは区別します。

現在の受入条件・実譜面基準・品質差・優先工程は [画像MusicXML品質受入](IMAGE-MUSICXML-ACCEPTANCE.md) に記載します。
合成63イベント＋明示補正8とexact7/64のXML/XSD成功は回帰証拠です。実譜面の同等品質の証拠には読み替えません。
画像入力からの実譜面品質受入はまだ未達であり、現在のglobal完了フラグをtrueにはしません。
環境・モデル設定・既存認証と通常main保存/readbackを維持し、元task/環境/work189を保持します。
未取得ce785c9は取得済み扱い・置換・再生成をしません。

最新の画像接続: 構築時の実System領域とStaff参照、12仮想Ledger曲線をPAGE/STEMS/SYMBOLSまで保持し、
543実glyph中心のSystem所属と796地点のStaff/null/曲線音高を原版比較した。
仮想曲線はpost-analysis callbackで保持し、後の物理Ledger削除やreportから作り直さない。
対象3件・fmt/clippy/build・63＋補正8/exact7/64 XML byte/XSD成功。
実所属543 glyph、176 cluster・最大23部品から原版7部品選択、788候補集合と遭遇順固定の再帰契約を比較し、2部品限定を解除した。
原版自身のHashSet順は再観測で53部品順/25試行順が変わり、候補集合は同じ。実譜面XML品質受入は未達。実画像CLI v16は166.87秒でpositive AlterHead linkガードへ進み、XMLは未出力。
古い全マスク観測は誤ったImageJ constructorによる全0バイトで無効と明示した。
実Bravura51ptの整数pointSize/4を保持し、消去余白を原版の2pxへ修正した。
正しい原版全マスク3,132,856バイト・黒82,837画素が一致。修正前の109画素差と強弱判定観測は保持した。
[証拠](../evidence/real-symbol-clusters/VALIDATION.json)・[次依存](../evidence/real-symbol-clusters/NEXT-DEPENDENCY.md)。

## 実装済みの具体的単位

- PNG 8-bit grayscale/PGM読取、原版の適応二値化・runs・classifier特徴量と149出力モデル推論。入出力形式・領域上限・全種類のページ処理は下表の部分対応です。
- 共有SIGの所有ID、glyph registry/cache、関係・削除履歴の保持、および保存された多数のGRID/HEADERS/BEAMS/HEADS/STEMSの比較単位。全SIG意味論の移植完了ではありません。
- 限定画像CLIで四分/二分音符と休符、第一下加線C4、複数小節・改行を認識し、nativeの結果からMusicXMLを生成。旧成功入力を保持しています。
- 今回の2部品四分休符の部品/union登録、RestInter・chord・LINKSへの接続。Rest/TimeWholeの非support・非frozen部分について原版の実画素overlapとexclusion選択を移植。

全回帰の件数・ソースSHAは段階ごとの検証要約で示します。通常workspace移行前のMANIFESTとLaura352heads/513exclusionsなどの報告は当時のソースSHAを保ち、新ソースで再実行した証拠に読み替えません。

最新の全回帰はsource `95e191b8ffaf5df56f66684b9891cf100bdd8794`で595成功です。後続checkpointでは対象検証を行い、595件を新ソースの結果に読み替えません。以下は過去の全回帰記録です。source `52943de`で583成功・0失敗・ignored 0。途中中断後は未完対象のみ再開し、176実行物を全照合。doc tests、対象fmt/clippy、別clone Linux build・20イベント・MusicXML4 XSDも通過。[検証記録](../evidence/musicxml-ordered-foundations/FULL-VALIDATION.json)。この後の説明だけのcommitは同じsrc/crates treeです。

続くordinary foundation overlap段階はsource `ea8c756`で対象37件・fmt/clippy成功。原版SIG11条件の実画素/core/Area/grade比較と、別clone Linux CLIの既存20+11イベント・MusicXML4 XSDを確認。競合hook画像ではnativeで新規overlap除外1件、2-system画像では0件です。原版の画像中SIG全件一致や新しい全回帰の証拠とは区別します。[今回の検証](../evidence/musicxml-foundation-overlap/VALIDATION.json)・[独立取得](../evidence/musicxml-foundation-overlap/READBACK.json)。

## 機能別の現在位置と欠落

| 原版機能・具体的依存 | 状態 | nativeの現在位置 / 残り |
|---|---|---|
| LOAD / `ImageLoading`, `Book`, `Sheet` | 部分 | grayscale PNG/PGM単画像・複数独立画像CLIあり。RGB/alpha/JPEG/TIFF/PDF・複数ページbook・原版画像変換/ページ選択は未接続。4M画素等は現ガードで、最終的な省略ではない。 |
| BINARY/SCALE / adaptive filter, `ScaleBuilder` | 部分 | 原版式・histogram/peak・多数kernelあり。一般画像の全設定/分解能/失敗状態と下流伝播を広げる必要がある。 |
| GRID / `GridBuilder`, `Staff`, `SystemInfo`, parts | 部分 | sections/filament/通常staff/system/parts・cue第二パスの比較あり。非zero skew、特殊staff、実スキャンの全工程一致は未検証。 |
| HEADERS / `HeadersBuilder`, clef/key/time関連クラス | 部分 | 音部記号・調号・拍子の探索/状態に既存部品と原版比較あり。画像→MusicXMLはG/F/Cの汎用意味変換を接続し、ヘ音/ハ音記号の原版成功画像各1件を追加比較。small/8va/8vbはkernel比較のみ。固定TimePairの4/4と3/4を接続済み。固定HEADER調号を実SIG/実glyphへ接続し、G major1画像のF♯を原版と一致確認済み。途中変更、他clef、一般調号と臨時記号、他拍子の意味伝播を接続する。 |
| STEM_SEEDS/BEAMS/LEDGERS / 各Builder | 部分 | 種/beam/ledgerの既存比較あり。単独beamと中間stump/二段sibling beam/前後hookのpositive画像経路を接続。一般hook/alien/cue/cross-staff beam、一般加線音高・共有staff全工程は未完。 |
| HEADS / `HeadsBuilder`, template catalog | 部分 | 24形catalog等のkernel、Laura/制御比較あり。24形全ページ伝播・別font/サイズ・特殊staff・実画像の全面比較は未完。 |
| STEMS / `StemBuilder`, `StemsStep`, head/beam linkers | 部分 | head由来画像stumps/chunks/items/glyph/cache/live SIGの接続あり。単独/兄弟beamからのitems/expand/profile4・中間profile1 glyph/cache/SIGとbeam-side supportを接続。前後hookのprofile0・実SIG/cleaner/exportを接続。8条件のsystem横断orphan判定と2-system画像の正の1削除・20イベントを比較。実free-glyph所属を維持したepilog bar領域seed再整理も接続。一般hook/alien/cross-system/cleaner/epilog、32sources/items制約の除去・検証が残る。 |
| REDUCTION/LINKS / `SigReducer`, `SIGraph`, `SymbolsLinker`, `InterCleaner` | 部分 | 共有SIG、一部head/stem cleaner、弱休符/空chord削除、Rest/TimeWhole overlap/exclusionあり。ordinary head/beamの方向・checkStems・tail link/侵入exclusionとcanonical cache削除、既存foundation整理の反復を接続。ordinaryの6チェック順・各チェック後contextualize/実所有weak purge・後続exclusion/epoch反復も接続。ordinary black/void headのanalyzeChords prolog/late（同音価HeadHead support/異音価不互換/未接続交差head exclusion）も接続。通常head/stem/beam/hook/barlineの実画素・head core・直線Area overlapをprolog前に接続し、GRID barlineの既存runsも解決。全Inter support、mirror/ensemble/header/特殊形など一般overlapの残り、small/percussion/stemlessのchord枝、frozen、LINKS各関係のconsistency/epilogは未完。原版に`reduceSymbols`というメソッドはなく、この経路は`reduceLinks`。 |
| TEXTS / `TextsStep`, Tesseract, words/sentences | 未対応 | 歌詞・題名/作曲者・tempo/メトロノーム・文字role認識、staff/partとの関連付け・OCR修正。原版のローカルOCR依存に沿う。外部AI設計に置換しない。 |
| MEASURES/CHORDS / `MeasuresBuilder`, `ChordsBuilder`, `MeasureFiller` | 部分 | 単staff/単stemの小節・chord形成あり。複段part、和音、stemless whole/breve、cross-staff、複数stem、repeat/double bar等の一般形成/補正が残る。 |
| CURVES / `CurvesStep`, slur/wedge/ending/rehearsal builders | 未対応 | 音楽上のslur/tie/wedge/ending/rehearsal認識・接続。nativeの`curves.rs`はGRID曲線補間であり、この工程の移植証拠ではない。 |
| SYMBOLS / `SymbolsFilter`, `SymbolsBuilder`, `InterFactory`, `DotFactory` | 部分 | classifier・原版7部品上限の再帰cluster・isolated/compound rest/time・実加線へのdot距離検査と普通の1旗→stem接続あり。全形のfactory/shape checks、真正augmentation/staccato/repeat dot、flag/accidental/articulation/dynamics/ornament/数字等の生成とlateChecksが残る。149分類出力を全記号の意味接続済みと扱わない。 |
| RHYTHMS / `SlotsRetriever`, `ChordsMapper`, `StackRhythm`, `MeasureRhythm`, `PageRhythm`, `Voices` | 部分 | 認識済み拍子に応じた完全小節の単声の二段beam十六分/beam・旗付き八分/四分/二分イベントをnativeで計算（全画像比較は4/4と3/4）。一般slot/voice割当、他beam/flag音価、付点/連符、tie、pickup/不足超過小節、複声部/複段/途中拍子変更の原版処理は未接続。 |
| PAGE/score / `ScoreReduction`, logical parts | 部分 | 単ページ改行・小節番号あり。複数ページ継続、part同定/結合、楽器/移調、跨systemの声部/slur/tie、repeat構造等の一般score統合が残る。 |
| MusicXML / `PartwiseBuilder`, original proxymusic | 部分 | native MusicXML4 writer、限定イベント・system break・part名、schema/reimport比較あり。一般notation/direction/lyric/instrument/layout/score属性と.mxl圧縮、多ページexportの原版対応が残る。原版XMLの流用は認識完成に数えない。 |
| プロジェクト保存・再開 / `Book`, `Sheet`, stage invalidation | 部分 | stage reportsとGit完全復元あり。native認識projectの永続化・読込・編集後の必要工程再実行、原版.omrとの対応/互換性は未対応。Gitソース保存とは別機能。 |
| 補正 / `InterController`, tasks/history, editors | 部分 | 下記CLI案の対象。GUIクラス内に存在することを理由に認識・補正ロジックを除外しない。 |
| パラメータ/処理選択 / CLI/constants, sheet/book settings | 部分 | 既存段階CLI/一部設定あり。原版の認識parameter、stage/page選択、再処理、出力設定への到達可能性を機能単位で移植する。 |
| ウィンドウ/キャンバス/パネル/メニュー/マウス・zoom描画 | GUIのみ除外 | 表示と入力UIだけを対象外とする。選択後の変更・validation・再計算は対象内。 |

## GUI内の補正をCLIで扱う方針（未実装）

原版5.11の`InterController`公開APIを実クラスから読み取り確認しました。`addInter`, `assignGlyph`, `removeInters`, `link/unlink`, `connect`, `buildKey`, `changeTime`, `changeVoiceId`, `splitChord/mergeChords`, `toggleTie`, `mergeSystem`, `toggleIndentation`, text/number/metronome変更、stack/page rhythm再処理、undo/redo/historyが対象です。`editInter/editObject`も意味を持つ形状変更部分は移植対象です。`setSheetEditor`など表示依存の接着だけを除外します。

実装案は、保存projectのglyph/Inter/Relation IDを指定する補正記録をCLIで適用し、原版tasksと同じ依存工程を無効化/再計算して再exportする方法です。原画像/project SHAと変更履歴を保持し、undo/redo、誤ID/古いprojectの拒否、トランザクションを備える方向です。これは未実装の方針であり、利用可能なコマンドがあるという主張ではありません。具体的オプション形式は原版task依存を調べてから確定します。補正基盤は全自動認識の完成まで後回しにせず、所有SIG・project永続化から段階的に接続します。

## 実用CLIへ広げる依存順

現在の直近工程はordinary foundationsの6チェック順・各weak purge・後続exclusion/epoch反復と通常headの`analyzeChords` prolog/lateです。通常foundationの実画素/core/直線Area overlapをprolog前へ接続しました。次の依存はmirror/ensemble/header/特殊形を含む一般overlapの残り・形/size互換性とLINKSです。HEADER・短音価の下記段階は履歴として保持し、未接続は各機能表で示します。

`key_state.rs`/`time_column.rs`の固定KeyInter/TimePairを`music_headers.rs`で実SIG/実glyphへ接続し、`music_symbols.rs`のPageCleanerへ伝播しています。`music_xml.rs`のalter=0・拍子4/4固定は撤去し、調号と選択済み拍子に由来する音高・小節期待長を使います。原版成功のG-major/F♯、固定4/4、固定3/4の画像を比較済みです。G/F/Cの音高/exportは汎用接続し、bass/alto画像も比較しました。次の切断箇所は他clefの全画像条件、cut/その他TimeWholeの全画像比較、臨時記号・途中調号/拍子変更、分数quarter単位のリズムです。原版`HeadInter.getAccidental/getMeasureAccidental/getAlteration`、`PageRhythm.populateTimeSignatures`、`PartwiseBuilder.processTimes/processKeys/processNote`の依存を順に接続します。

1. **今回のcompound rest/SIG checkpointを固定**。実装済み部分、画像全工程比較、保存XML段階比較、条件指定競合kernelを分けて保存する。
2. **直近の未接続はHEADERの意味伝播**。既存clef/key/time部品を実measure/音高・臨時記号/拍子状態とexportへ繋ぐ。調号と固定TimePairの実画像比較は完了。固定COMMON_TIMEも追加接続。bass/altoの実音高/exportを接続。次はcut/他TimeWhole、他clef条件や短音価に必要なbeam/STEMS・臨時記号を依存順に接続する。調号/accidentalの原版依存が必要なら最小部分を合わせて接続する。
3. **positive beam/head STEMS、一般chordと音価**。自動items/glyph/beam接続→cache/SIG→cleaner/epilogを閉じ、beam/flag・whole/breve・一般加線へ広げる。単なる条件指定APIの追加ではなく実画像CLIまで通す。
4. **generic SYMBOLS/LINKSとlateChecks**。多部品候補、全Inter factory、support grade/frozen/競合反復、dot/accidental等の関係を原版に沿って接続。実際に必要なものから小段階にする。
5. **CURVES/TEXTSと一般MEASURES/CHORDS/RHYTHMS**。原版stage順を守り、tie/slur/連符/lyrics等をLINKS・slot/voiceへ供給。複声部・複段part・拍子変更・pickup等を段階的に画像→MusicXMLへ通す。
6. **PAGE/score/export全般と補正project**。複数ページ/楽器/移調/repeat/文字・direction/notation/.mxlを接続。project保存/再開・補正CLIは所有SIGに依存する最小部分から途中段階でも着手する。
7. **入力/前処理と実譜面条件を各段階に合わせ拡張**。RGB/PDF/複数ページ、非zero skew、別font/scale、特殊staff、実スキャンも最終対象に保持する。一度に全画像条件へ拡張せず、関連する原版成功例で検証する。

順序は実際のJava依存と検証可能な保存入力により細分化/調整します。各段階は、実行可能Linux CLI・新しい対応機能・原画像/原版比較・実行例・未対応・ソースSHA・検証/復元receiptを小チェックポイントに保存し、ユーザーが試せる状態にします。原版も失敗する入力の独自推測による成功化は行いません。小段階成功を全機能完了と呼びません。

## 現在試せる経路

```sh
# 復元ソースでの既存CLI
cargo run --offline --locked --bin audiveris-musicxml -- \
  fixtures/official-image-musicxml-compound/compound-quarter-rest.png \
  compound-quarter-rest.musicxml compound-quarter-rest.report.json
```

出力先は未存在にします。耐久保存したLinux実行物ならJava/Cargo無しで同梱入力を実行できます。配布tarとSHA・全回帰結果はcompound checkpointのMANIFEST/CLI-GUIDEが正本です。既存限定入力の詳細は[MUSICXML-LIMITED.md](MUSICXML-LIMITED.md)。その制限は現在の受付境界であり、最終機能削除の予定ではありません。

以前の40〜120実作業時間はJava移植込みの限定縦断経路に対する低確度の暫定値で、全原版同等性の工数ではありません。18〜60hの未算定仮枠も確約ではありません。全機能同等性の追加工数は細部の依存・検証範囲を調査してから再評価します。


## HEADER調号の縦断追加（2026-10-05）

原版5.11が正しく処理した通常五線・Bravura44pt・ト音記号・G major・4/4・単声2小節の1画像を追加確認しました。固定KeyInter/KeyAlterInterの実glyphとContainment/ClefKeyRelationを所有SIGへ取り込み、`HeadInter.getAlteration`の調号参照と`PartwiseBuilder.processKeys/processNote`に沿って8音（F♯5を含む）をnative MusicXMLへ伝播します。臨時記号・tieは未接続です。キー未掲出の小節でcontextを保つ原版export契約を保持します。

必要だった`SigReducer.checkLedgers/ledgerHasHeadOrLedger`を通常五線・system内1staffに限定して接続し、sharpの孤立加線を消します。元画像の原版export成功とは別に、保存STEMS XML再読込→原版reduceFoundationsの孤立加線削除を観測しました。物理glyph/cache/削除ID履歴は保持し、既存C4加線は残します。複数staffのshared-ledger補正は未接続です。

固定HEADERの4/4 TimePairは次の1制御画像で所有SIG・数字glyph・拍数・exportを接続済みです。次の具体的未接続は他clefの音高/exportと分数拍・途中変更です。cut/その他TimeWholeの全画像比較も残ります。SYMBOLSで初めて認識された調号、cancel/途中調号変更、一般臨時記号はその後に接続します。これらを最終機能から省略しません。HEADS/STEMS/MusicXML/epilog完了フラグはfalseです。


続けて、同じG-major譜面のTime数字を原版HEADER探索範囲へ置いた1制御画像で、固定4/4 TimePairのTOP/BOTTOM実glyph、Containment/TimeTopBottomRelation、小節拍数とMusicXMLを接続しました。原版独立画像exportの8音とnativeが一致。最後の数字glyphが不正な場合のGraph/registry不変も確認します。他拍子を4/4に置換して成功扱いにせず、その他のHEADER timeは明示的に未対応です。移行版552全回帰とこの追加変更の対象回帰はソース時点を分けて記録します。

## 固定3/4拍子の縦断追加（2026-10-05）

1つの新しい2小節G-major画像を原版5.11の変更なし画像pipelineでexportし、6音 B4 G4 E4 | B4 F♯5 D5 と3/4をnativeで一致確認します。TimePair/TimeNumberのshape/valueとContainment/TimeTopBottomRelationを検査し、数字の実glyphを所有SIG/registryへ原子的に登録します。`PageRhythm.populateTimeSignatures`→`TimeRational.getValue`に沿って期待小節長を分子/分母から計算し、次小節へ継承。元のOMRに保存された3/4小節長・onsetも別途比較します。四分/二分単位で表現できない拍子長、途中拍子変更、未掲出拍子の推測は拒否を維持。その他の整数quarter拍子の経路は一般化したものの、全画像比較済みとするのは4/4と今回の3/4のみです。全機能省略なしの目標とglobal完了falseは変えません。例・コマンドは[MUSICXML-LIMITED.md](MUSICXML-LIMITED.md)。

## 固定COMMON_TIMEの縦断追加（2026-10-05）

3/4の次段階として1つのcommon-time/G-major画像を原版5.11の同一画像処理で比較し、8音と`<time symbol="common">`をnativeから出力します。`TimeWholeInter`の原版shape→分子/分母を既存TimeColumnから再利用し、選択済み固定glyphを所有SIG/registryへ原子的に登録。`PageCleaner.visit(TimeWholeInter)`と`SigReducer.reduce`のHEADER Inter除外に沿って接続し、`PartwiseBuilder.processTime`のCOMMON/CUT表記をwriterへ伝播しました。画像全工程の新比較はCOMMON_TIME1件であり、cut/その他TimeWholeのmapping接続を全画像検証済みと数えません。台帳は全機能省略なし・他clef/拍子変更/臨時記号/複声部など未完を保持します。

このHEADERまとまりで全workspace554回帰を一度実施し全件成功。fmt、全target clippy、Linux CLI、MusicXML4 XSDも成功。過去の552/549等の報告は当時のソース記録として保持し、今回の全回帰と置換しません。通常ソースGit保存とSHA/tree readbackを用い、新規bundleや全ファイルhash一覧は生成しません。


## HEADER clef意味伝播とfactory-null依存（2026-10-05）

G-clef固定ガードとwriterのsign=G固定を撤去し、既存`ClefInter.noteStepOf/octaveOf`を所有measureの実ClefInterへ接続。`PartwiseBuilder.buildClef/areEqual/processClef`に沿い、sign/line/octave-changeを出力し、同じ意味の改行HEADERを重複出力しません。音高を成功例のリストから供給しません。Bass/Alto各1画像の8音・2小節が原版の同一画像exportと一致しました。12条件のexport kernelと既存180音高条件は全画像一致と別に扱います。

Altoで必要だった左端残留線のARPEGGIATO候補には、原版`InterFactory`→`ArpeggiatoInter.lookupLink/createValidAdded`の接続先無しnull分岐を移植。既存profile1の2.5 interline右側矩形にhead chordが無ければ記号を作らず、可能性があれば未対応として拒否します。原版kernelのnull/接触/1px重複境界を比較。正のarpeggiato接続・grade・exportは未完です。

次の具体的未接続は、短音価を支える正のbeam/flagとSTEMS/LINKS、臨時記号・途中調号/拍子/clef、分数拍の単声rhythmです。cut/他HEADER形・ledger・多声部/多段/移調・保存/補正・他入力条件も全機能台帳に残します。途中clef変更や打楽器は現在拒否し、全機能完了フラグfalseを維持します。実行例は[MUSICXML-LIMITED.md](MUSICXML-LIMITED.md#recognized-bassalto-header--musicxml)。直前全554は51f8d44での結果で、今回は対象検証を行い不必要に全suiteを反復しません。


## 旗付き八分音符・分数拍の小単位（2026-10-05）

`AbstractFlagInter.lookupLink/createValidAdded`と`FlagStemRelation`を、分類済み実glyph・所有SIGのstem/headへ接続。旗の位置/foot・stem方向・gap impacts・first-valid順を原版から移し、HeadStem/FlagStemの元support計算と弱旗purgeを実行します。`AbstractChordInter.getDurationSansDotOrTuplet`の接続数に由来する1/8音価を、整数eighth tickによる単声onset・小節検査とMusicXML divisionsへ伝播。原版成功1画像で八分2音/四分7音、旗glyph/保存grade/関係とMusicXML意味を比較しました。これはbeamの無い入力で、正のbeam一致とは区別します。

旗checkpoint時点の次の切断箇所は`BeamLinker.BLinker.VLinker`のbeam-origin items収集・expand/createStem・頭との接続順、`SigReducer.checkBeamsHaveBothStems/checkHooksHaveStem`と`StemsRetriever.finalizeBeams`、`AbstractChordInter.getBeams`と`PartwiseBuilder`のbeam begin/continue/end出力です。既存head-origin builder/kernelは部品として再利用できますが、head-onlyガード撤去で代用しません。旗の下向き/小型/複数・一般リズムや原版全機能も台帳の未完として保持します。


## 正の単独beamと八分音価（2026-10-05）

保存済み1画像を原版成功exportと比較し、画像由来beam-origin builder→
profile4/実cache/BeamStem・HeadStem→beam-aware support/競合整理→実chord membership→
分数音価・onset・MusicXML番号1 begin/endまで接続。9イベントの意味が一致しXSDも成功。
途中の2builder/items/選択glyph/grade/関係、6人工stem制御、原版HEADS reduction、
PageCleaner area rasterとBeamHeadCleanerはそれぞれ別の比較境界です。
全BEAMS/STEMS/SIG一致とは区別します。旧成功例と不足小節の拒否を保持。

次は中間stumpのside-first/失敗時分岐と正のcontinue画像比較、
sibling/multiple-level/hook・alien closer limit・cross-staff/systemと所有接続、
下向き/複数旗、臨時記号・付点lateChecks・一般rhythmへ依存順で進める必要があります。
現在は1system1standard beam/1part、水平restored eraserに限定した小単位であり、
原版非UI全機能の最終目標と省略なし方針は維持。試す手順は
[MUSICXML-LIMITED.md](MUSICXML-LIMITED.md#single-standard-beam-eighth-notes--musicxml)。

この実装treeの全563一意テストを分割回帰で確認し、Cargo全一覧との照合で漏れ・重複計上なし。
途中停止は検証要約へ別記し、単一無停止の全suiteとは区別。fmt/clippy、別cloneのLinux CLI
ビルド・同一画像9イベント・MusicXML4 XSDも成功。
[最小検証要約](../evidence/musicxml-beam-eighth/VALIDATION.json)。


## 中間stumpとcontinue（2026-10-05）

原版成功の別入力1画像で、4連八分の左右→中間stump順、実cache/SIGのCENTER支持、
10イベントの音高・音価・onsetとbegin/continue/continue/endを画像CLIまで接続・比較。
水平fractional eraserをJava2Dのquarter-pixel正規化から接続し、数値境界8条件を別検証。
対象回帰とfmt/clippyを実施。全suiteはこの小段階で反復していません。
次は兄弟beamのfilter/items/linkSiblings、共有stemと複数段beamの音価/export。
[実行例・範囲](MUSICXML-LIMITED.md#intermediate-beam-stumps-and-continue)。
全機能同等性は未完、完了フラグfalseを保持します。


## 兄弟二段beamと十六分音価（2026-10-05）

通常五線・ト音記号/C major4/4単声の別入力1件を原版と比較。兄弟beamのfull
LinkerItem→共有stem/cache/SIG、全beamの左右接続→中間stump、linked状態、
BeamBeam支持の係数3、chord tailからのbeam番号順、実接続数由来の1/16音価を
画像CLIまで接続しました。11イベント・onset・beam1/2 begin/continue/endが一致しXSD成功。
HEADS保存状態の8builder/items/lengthsと4共有stem grade/8関係支持の比較は、
原版の同一画像RHYTHMS/export比較と別に記録。原版全SIG/全工程一致とはしません。

次の未接続はalien closer limit/別group、hookと競合hook、cross-staff/system、
複数旗/追加beam段、一般rhythm・補正・cleaner/epilog。台帳の最終scopeは省略なし。
[実行手順・現在の限定](MUSICXML-LIMITED.md#two-sibling-beams--seventeenth-notes)。


回帰記録: 321abfcで単一全workspace565件成功（失敗/ignored0、Laura352/513を含む）。
その後、原版`Inters.byReverseWidth`の整数bounds幅・同幅stable順を境界3条件で
修正し、対象5件とfmt/clippyが成功。最新版全566件を再実行済みとはしません。
旧証跡・fixtures・checkpointsのGit内容は保持。次の具体的な切断点は
[原版依存一覧](../evidence/musicxml-beam-subdivisions/NEXT-DEPENDENCY.md)に記録。
## 複数の通常beam groupと探索制限（2026-10-05）

原版`BeamLinker.VLinker.getCloserLimit`を実所有beamの境界・grade・groupから接続。
別group/良好beam/線分交差/近隣side例外/方向別borderソートを原版の17数値条件で比較。
PageCleanerの薄いAA-off contourでは、端点正規化で同じ矩形になるfractional slopeも
原版画素と比較して接続しました。任意傾斜を矩形の塗り潰しで通す処理ではありません。

新入力1件の2連八分×2group→実cache/HeadStem/BeamStem→独立begin/end→MusicXMLで
10イベントの音高・音価・onset・voice・beamと連桁音符のstem方向を原版と比較。
同入力の4 HEADS builders/items/lengths/glyph比較と数値kernel比較は別の証跡です。
画像上のpositive alien limitや全SIG/全MusicXML描画属性一致を確認したとはしません。

対象検証、98 library tests、旧beam/二分休符、Laura352/513、fmt/clippyが成功。
直近の全565回帰と後続境界検証は当時のソースに属し、今回は全workspaceを反復せず。
次はhookのprofile/競合整理/typed SIG/`checkHooksHaveStem`/forward-backward export。
cross-staff/system、alias、一般rhythm・補正・cleaner/epilog等も未完。
UI以外全機能同等性の最終目標を維持し、HEADS/STEMS等の完了フラグはfalse。
[CLI例・範囲](MUSICXML-LIMITED.md#separate-ordinary-beam-groups--musicxml)。


## 前後hookからMusicXMLへの接続（2026-10-05 America/Los_Angeles）

1件の独立合成入力で、原版5.11成功の11イベント（十六分4音＋四分7音）を
画像CLIで比較。実beam/hookのbuilder→profile0 hook／profile4通常side／
profile1中間stump→実cache/SIG→hook片側stem検査→PageCleaner→ChordStem→
音価・beam方向・MusicXMLまで接続しました。入力ごとの座標分岐はありません。
原版`BeamLinker.linkSides`、`AbstractBeamInter.getCompetingHook`、
`SigReducer.checkHooksHaveStem`、`PartwiseBuilder.processBeam`が対応依存です。

Exclusion＋同一実glyphの競合は、通常beamの両端接続時だけhookを削除する
原版4条件を別のSIG条件比較で確認。stemなし/CENTER/LEFT/RIGHTのhook保持・削除
4条件、実5領域の原Java2D消去範囲も別に比較しています。今回の画像のglyphは
全て別であり、同一glyph競合の画像全工程一致はまだ主張しません。

次は一般beam groupの所有/競合、追加beam段数・flag、付点/連符の既存factory・
lateChecks・Rational/rhythm依存を、原版成功画像で試せる単位に接続します。
最終目標は原版非UI全機能の同等性で、一般cleaner/epilog、跨staff/system、
非zero skew・実スキャン・別font・特殊staff等も未完として保持します。
[CLI入力例・手順](MUSICXML-LIMITED.md#forwardbackward-secondary-beam-hooks)。

## 所有glyphに基づくsystem横断Finalizer（2026-10-06）

原版`StemsRetriever.Finalizer`の隣接system順・整数bounds中心・staff外側の厳密判定・
同一の物理glyph・最初のBeamInter一致を、実所有GRID/registry/SIGへ接続。
原版が返す最初の相手がorphanなら後続のlinked相手へ飛ばしません。
削除後は既存group callbackとcache同一性を保ち、StemsStep順にsystemごと支持を再計算。
保存HEADSページに宣言した原版SIG8条件が一致しました。正の削除はこの条件比較であり、
画像全自動での正のcross-system orphan一致を意味しません。

追加した2-system連桁画像は原版20イベント/4小節に成功。nativeはHEADS未収録の
Bravura47ptで停止し、両者のhead幅14px/高さ12px/47pt算出は一致しています。
次はその正確なテンプレート・伝播を接続。全epilog、実スキャン、特殊staff、
非zero skew、全24形ページ等は未完、完了フラグfalse。
[検証・次依存](../evidence/musicxml-cross-system/VALIDATION.json)。

## 2-system連桁を47ptからMusicXMLまで接続（2026-10-06）

原版・nativeで同じhead幅14px/高さ12px→Bravura47ptとなる入力について、
原版TemplateFactoryから普通のoval4形を正確に収録。測定済みcatalogを
HEADS→STEMS→exportへ伝播し、サイズ代用・画像座標分岐は行いません。
以前の水平envelopeによる接続前候補削除を、移植済み実VLinkerの空/正候補の
一般スケジューリング→全system接続→Finalizer→端点reductionの原版順へ置換。
原版の同画像HEADS→process→epilogはbeam3件から2件となり、
上段のstemなし同glyph候補1件を削除。nativeの実registry glyphも一致しました。
原版のフル画像exportと20イベント/4小節の音高・音価・onset・voice・beam・
連桁音符stem方向が一致し、双方MusicXML4 XSD成功。全SIG/描画属性一致ではありません。

8条件の宣言SIG比較と、この画像からの正のorphan削除比較は別の証跡です。
一般cleaner/epilog、非zero skew・実スキャン・特殊staff・24形全ページ等は未完。
原版非UI全機能を省略しない最終目標・完了フラグfalseを維持。
[実行例](MUSICXML-LIMITED.md#two-system-beams-with-measured-head-font-and-owned-epilog)・
[検証](../evidence/musicxml-cross-system/IMAGE-VALIDATION.json)。

検証区切り: native source `c6f03c5` / src tree `048f9f41bc7309d85638aea50c7058c2100410e1`で
単一全workspace 577件成功、失敗/ignored0、fmt・all-target clippy成功。
原版観測adapterはStemsStep同様にepilogでsystemごと新retrieverを生成して再確認し、
beam/glyph/接続の観測結果は不変。native sourceは変更していません。

## STEMS epilogのvertical-bar seed再整理（2026-10-06）

原版 `StemsRetriever.finalizeBeams` のfreshなsystem seed取得と
`purgeNoStemSeeds` を画像CLIへ接続。systemのfree glyph所属をHEADSから
実所有状態で引き継ぎ、live SIGのBarConnectionから上・下・間の領域を再取得します。
重なり比は原版どおりmin面積を分母とし、strict >0.25です。
候補リストだけを除外し、canonical glyph/group/SIG/cacheは削除しません。

既存Lauraの原版保存HEADSを新たに観測し、上段98→97・下段202→200、
27領域と既存原版survivor全件の一致、system所属の保持を確認しました。
保存状態の比較であり、Laura全STEMSや複段MusicXMLの完成ではありません。
既存2system4小節20イベントの画像→MusicXML回帰を保持。
斜めbar領域、一般SIG reduction/cleaner、SmallBeam・複声部などは継続対象です。
[小さな検証要約](../evidence/musicxml-seed-reduction/SEED-VALIDATION.json)。

## ordinary STEMS consistencyとcache削除（2026-10-06）

`StemInter.computeDirection` のhead/beam全connectionをsource contextual grade順に
参照する部分と、`SigReducer.checkStems` のheadなしstem削除・tail側head link除去・
侵入時exclusionを接続。SIG削除と同じcanonical cached objectのremoved状態を
同時に更新し、physical glyphは保持します。構造変更後は既存head/beam/ledger
チェックへ戻り、単発の整理だけで未接続headを残さないようにしています。

原版5つの宣言SIG条件（headなし/上下優先/beam優先/方向不明）とnativeを比較。
実画像所有cache上の宣言head-loss/原子的な失敗検証は、画像そのものの誤認修正とは
別です。既存の8-head foundation、11イベント競合例と2system20イベントは
対象回帰で保持。新記号や新入力条件の対応完成は宣言しません。

一般reduction全順序・weak stemの初回purge callback・chord互換性・frozen/lateChecks、
lyricsに必要なInterCleaner/Partの所属更新は未接続です。
[原版条件の要約](../evidence/musicxml-seed-reduction/STEM-VALIDATION.json) /
[次の具体的な接続箇所](../evidence/musicxml-seed-reduction/NEXT-DEPENDENCY.md)。

## owned weak-stem initial purge（2026-10-06）

原版`SIGraph.deleteWeakInters`の初回選択を実所有SIGでstageし、削除するstemの
canonical cache objectを全て検証・lockした後、SIGと同じobjectのremovedを更新。
physical glyph保持・再登録拒否・所有情報欠落時の原子的拒否まで接続しました。
既存原版createStem grade約0.301、exact0.5、frozenの3宣言条件と比較。
これは画像そのものから弱いstemが生じる全工程比較とは区別します。

前節で未接続だった初回weak stem callbackはこの段階で接続済みです。
一般reductionの全adapter順序・各チェック間のcontextualizeAndPurge、
overlap/chord/frozen/lateChecksとlyrics/PartのInterCleaner依存は引き続き未完。
全580回帰はsource`c270552`で確認済み、追加変更は対象検証を別記しています。
[小さな原版/実所有検証記録](../evidence/musicxml-seed-reduction/WEAK-PURGE-VALIDATION.json)。

## 原版foundation順序と各weak purge（今回）

画像CLIのhead-only prefixと後付け安定化loopを置き換え、原版
`AdapterForFoundations.checkConsistencies` のending-head→head→hook→beam→ledger→stemを
個別に実行。各チェック直後にcontextualize/weak purgeを実SIG/cacheへ適用し、
modificationがなくなるまで反復、その後にexclusion整理、weak削除/競合削除があれば
次epochへ戻ります。診断CLIの過去のhead-only観測境界は保持しています。

原版の宣言3Inter連鎖（リンク除去→弱head削除→headなしstem削除→次passのhook削除）は
3pass・変更数2→1→0。各メソッドの観測と実Adapter呼出を別に実行し一致確認。
nativeの実画像ownerへ宣言条件を追加した検証は、画像からこの条件が生じたという
主張と区別します。新画像fixtureを増やさず、既存の2system4小節20イベント例を使います。

この順序接続checkpoint時点では、接続済みの処理単位を原版工程全体の完成とは数えていません。
当時未接続の`analyzeChords` prolog/lateは次節で通常head部分を接続。一般overlap、
全形support、merged staff/frozen/implicitと
LINKS各記号、lyrics/PartのInterCleaner、完全epilogは未完。64回の各反復budgetも
現ガードであり全同等性ではありません。[次の具体的依存](../evidence/musicxml-seed-reduction/NEXT-DEPENDENCY.md)。

## ordinary chord互換性とsupportのprolog/late（今回）

原版`SigReducer.analyzeChords`の通常black/void head部分を接続。intrinsic gradeが
good（>=0.35）のstemをgrade順に処理し、medianが交差してbox IOU>=0.02の
未接続headへOVERLAP、同stemの異音価headへINCOMPATIBLE、同音価head同士へ
HeadHeadRelationを登録します。原版のduration bucket/edge/member順、canonical ID、
既存support/exclusionを相互に上書きしない規則を保持します。

HeadHeadRelationの原版係数0.75/default grade1をSIG contextualizationに接続。
画像CLIは最初のweak purge前にprologを実行し、lateにはanalyzeChords→exclusion整理→
contextualize/purgeを配置。古い診断CLIの観測境界は維持。単一入力の記号位置で
分岐せず実SIGの接続とgeometry/shapeに基づきます。

原版2宣言SIG条件（同音価/異音価、いずれも未接続の交差headを含む）の
新関係・全contextual grade・二重登録しない結果がnativeと一致しました。
これは同じ実画像から和音を全自動認識・exportできたという主張ではありません。
small/percussion/stemless/cueのsize/duration枝、一般overlapと全cleaner/epilogは
依然未完です。全非UI機能は最終目標に保持します。

## HeadStemsCleanerの実所有接続削除（今回）

原版の最初のtyped relation取得と`HeadStemRelation.removed`を接続。
STEM全体の再同期をやめ、選択した実edgeだけを削除して両endpointのabnormalを
再計算します。manual/frozenの属性、他head/beamの接続、physical registryと
canonical cache objectは維持。元版6宣言SIG条件で選択順と2回の処理結果を比較し、
実画像ownerへ宣言追加した接続でもcache保持・途中不正入力の原子的拒否を確認しました。
この宣言条件の検証を画像全工程の正の複数stem認識とは区別しています。

原版自動parallel条件1件は削除済みedge参照で停止しました。再試行せず、nativeの
自動parallel拒否を維持。他方の自動parallel条件は未観測です。対象33テストと
fmt/workspace-all-target clippy成功。全583は過去source52943deのまま。
[検証](../evidence/musicxml-head-cleaner-lifecycle/VALIDATION.json)。
mirror/ensemble/特殊head、全support/Part/lyrics所有、完全cleaner/epilogは引き続き
未完であり、原版非UI機能の省略予定はありません。

この単位の実装sourceは`e532ae2`。独立fetch後のroot/src/crates treeが一致し、
Linux CLIで既存20イベント・11イベントの原版一致とMusicXML4 XSDを確認しました。
XML byte hashも前成功版と同一。新しい画像対応や全SIG同等性の宣言ではありません。
[独立取得照合](../evidence/musicxml-head-cleaner-lifecycle/READBACK.json)。

## 特殊headのoverlapサイズ判定（今回）

原版`Shape.isSmallHead`の4形分類を共有し、認識済み24形のoverlap判定に接続。
small head/stemだけが接触免除から外れ、実画素/core判定と既存support優先は維持。
原版6宣言SIG条件でsmall4形・標準cross/breve・support有無・contextual gradeを比較。
これは特殊headページを画像からMusicXMLへ処理できたという結果ではありません。
後続size/duration/chord/所有処理は未完・guard維持、既存画像CLI範囲は変更なし。
[検証](../evidence/musicxml-special-head-overlap/VALIDATION.json)。

特殊head判定の実装source`3f76b49`で対象12・fmt/clippyと独立取得Linux CLIを確認。
既存20＋11イベント／MusicXML4 XSD／XML byte hashを維持、root/src/crates tree一致。
[readback](../evidence/musicxml-special-head-overlap/READBACK.json)。

## typed mirrorとensemble実memberの競合判定（今回）

原版の`getMirror/getChord`・Containment・bottom-up member順を実SIGから取得。
mirror先のhead/chord/notesを競合対象から外し、ensembleのoverlapは実memberの
両方向判定とnoSupportで判断します。box間の空白や自身のmemberは競合しません。
原版7宣言条件を比較。MirrorRelationはSupport（係数0）であり、member間競合を
抑止する一方gradeは増やさない点を追加比較で検出・修正しました。
実画像ownerへ宣言したtyped mirrorでもregistry/cache identityとgradeを保持。
自動mirror生成や全画像SIG一致の主張とは区別します。

stem付きchordの境界形状、全ensemble/所有callback、特殊headのduration/size/chord、
Part/lyricsのcleaner・LINKS epilogは未完。最終非UI機能の省略予定と全体完了宣言はなし。
[検証](../evidence/musicxml-mirror-ensemble-overlap/VALIDATION.json)。

実装source`00ee541`で全588（176binary＋doc-test7組、失敗/ignore0）を一度実行し、
fmtとworkspace/all-target clippyも成功。独立fetch/buildのLinux CLIで20＋11イベント、
原版音高/音価/連桁等・既存XML byte hash・MusicXML4 XSDを維持。root/src/crates tree一致。
[全回帰](../evidence/musicxml-mirror-ensemble-overlap/FULL-VALIDATION.json)、
[readback](../evidence/musicxml-mirror-ensemble-overlap/READBACK.json)。
次の特殊head音価/サイズ整理は原版8条件・24形の観測のみ保存済みで、Rust未接続。

## Intrinsic head duration / beam-size consistency

Original `SigReducer.analyzeChords` now groups heads by intrinsic duration,
not shape name. Size exclusions precede same-duration support; actual BeamStem
edges supply standard/small beam mutual and head-size exclusions. Eight original
declared SIG conditions and24 Shape duration/size/order observations agree.
Existing ordinary prolog/late, atomic unknown/frozen refusal and production
special-head/small-beam builder guards remain. The observed original default
allowSmallHeadOnStandardBeam=false is supported; its nondefault setting remains.

This expands a generic original processing unit, not special-head raster-page
admission or full MusicXML/cleaner parity. Targeted regressions/fmt/clippy are used;
last full588 is source`00ee541`, before this change. The existing two-system image
and competing-hook CLI examples remain the image checks. Next: original
stem-bound HeadChord bounds/leading-note/tail and actual ensemble ownership.
[Validation](../evidence/musicxml-special-chord-consistency/VALIDATION.json).

Source`853b19d`:10 targeted checks passed, including Laura352 heads/513 exclusions;
independently fetched Linux build/20+11 image events/prior exact MusicXML bytes
and native-original MusicXML4 XSD passed. Root/src/crates trees agree.
[Readback](../evidence/musicxml-special-chord-consistency/READBACK.json).

## Stem-bound HeadChord geometry / live owner

Original `HeadChordInter.getBounds/getLeadingNote/computeLocations` now supplies
actual MEASURES/CHORDS and generic overlap ownership. Member bounds are united
and only the stem tail point is added; standard integer center, bottom-up members
and strict distance-tie order are retained. Four declared original controls match.
The former native chord envelope held heads alone. Beam ordering now uses the
original head location rather than the enlarged chord-box center; near-head rest
admission continues to check actual heads. Real Containment/ChordStem and canonical
cache identities are retained and validated before committing the continuation.

Empty-member original location getters throw; native does not invent a note
location. Stem-only bounds are compared separately. Complete ensemble grade /
addition-removal / Part-lyrics callbacks and full cleaner/epilog remain pending.
No full stage flag is enabled. Existing two-system/competing-hook CLI examples
remain usable;17 focused checks include the11-test MusicXML regression file.
[Validation](../evidence/musicxml-stem-chord-geometry/VALIDATION.json).

Source`593425b`:17 target regressions/fmt/clippy and independently fetched Linux
20+11 original-matching image events/prior XML bytes/MusicXML4 XSD passed.
Fetched root/src/crates trees agree; full588 remains earlier00ee541.
[Readback](../evidence/musicxml-stem-chord-geometry/READBACK.json).

## Actual HeadChord identity callbacks

Typed Containment/ChordStem additions/removals now invalidate the registered
HeadChord member order, original bounds/leading note/head-tail locations and
mean member best grade. Original setGrade leaves a computed contextual grade
intact. Six ordered original relation operations agree; malformed later edges
refuse atomically and repeated deletion retains original idempotence. Vertices,
physical glyphs and cached stem identities are preserved. Real MusicMeasures
construction invokes additions; uncached music vertex cleanup uses identity
removal callbacks. No public selected-member/grade recognition API is added.

Relation deletion leaves an empty chord with null bounds/grade, as original.
Cached extensive head/stem deletion, dying containers, RestChord/other ensembles,
Part/stack/lyrics and complete cleaner/epilog remain pending. The existing
image CLI examples remain the image comparison boundary. Six target checks/fmt/
clippy passed; full regression and fetched CLI evidence are recorded separately.
[Validation](../evidence/musicxml-chord-containment-lifecycle/VALIDATION.json).

Source`322135d` passed all590 tests (176 binaries+7 empty doc-test groups),0
failures/ignored, plus fmt/workspace-all-target-clippy. Independently fetched
Linux build/20+11 image events, prior XML byte hashes and native-original
MusicXML4 XSD passed; root/src/crates trees agree. All full-stage flags remain false.
[Full](../evidence/musicxml-chord-containment-lifecycle/FULL-VALIDATION.json),
[readback](../evidence/musicxml-chord-containment-lifecycle/READBACK.json).

Next original RestInter proximity5 conditions show the remaining head-only guard
difference at stem tail. Its standalone Rust prototype agrees, but CLI connection
is pending; the recorded4 RestChord ownership controls are original-only.
[Prototype boundary](../evidence/musicxml-rest-chord-admission/PROTOTYPE.json).

### Rest admission follows the actual chord envelope

`RestInter.createValid` now uses the actual chord bounds, including the stem tail,
rather than the old head-only union. Its half-open intersection and strict
center2D minimum-dx boundary are retained. The original five declared admission
conditions agree; the new native test first reproduced the tail mismatch before
the fix. This is connected to image rest admission, not a new classifier or
notation expansion. Targeted14 tests/fmt/all-target clippy pass; the last full590
run belongs to source`322135d`, before this change. Existing image MusicXML
regressions pass. RestChord vertex ownership, other ensembles and complete
cleaner/epilog remain pending.
[Validation](../evidence/musicxml-rest-chord-admission/VALIDATION.json).

### RestChord actual member and vertex lifecycle

`RestChordInter` now uses the same `AbstractChordInter` Containment invalidation
as head chords: bottom-up actual members, highest leading note, union bounds,
head/tail center and mean member best grade; existing contextual grade survives.
`AbstractInter.remove(true)` is connected for uncached Rest/RestChord ownership:
partial removal refreshes the surviving chord, last-member vertex deletion
deletes its dying chord, extensive chord removal deletes only unshared members.
Direct relation deletion remains distinct from vertex deletion. Stable vertex IDs,
glyphs and cached stems remain retained. This is the actual Symbols/Links graph
path, not a new caller-selected-ID API. Four original registered-SIG conditions
agree; existing weak-rest cleanup and image MusicXML regressions are checked
separately. Part/measure/stack/staff/lyrics callback bookkeeping, cached extensive
head/stem ownership, interleaved BeamRest and complete cleaner/epilog remain
pending. All global completion flags remain false.
[Validation](../evidence/musicxml-rest-chord-lifecycle/VALIDATION.json).

Existing runnable regression (choose unused output paths):

```sh
./target/debug/audiveris-musicxml fixtures/official-image-musicxml-cross-system/beam-two-systems.png /tmp/rest-owner.musicxml /tmp/rest-owner.json
```

This20-event image retains the earlier original MusicXML agreement. It does not
claim that all four declared ownership branches are generated by raster input.

### Pre-rhythm note/chord container synchronization

After LINKS reduction, native uses actual live `Containment`/`ChordStemRelation`
and tombstones to refresh its retained measure chord list, member metadata and
geometry. This follows `AbstractChordInter.invalidateCache/remove` and
`Measure.addInter/removeInter`; partial deletion no longer leaves copied bounds
or removed chord members. The stage report at `limited_links.containers` (CLI JSON:
`symbols.limited_links.containers`) also projects
original `AbstractNoteInter.added/remove` staff membership from the live SIG in
registration order. It is a derived projection, not an independent Rust Staff
object or complete editing implementation. Stale identity/geometry is refused
atomically. Live empty chords from direct relation editing remain pending.

Four saved original removal conditions now also match actual Staff notes and
Measure RestChord registration. The same-image weak-rest regression checks that
deleted IDs disappear from containers and cached stem objects remain identical.
Existing image MusicXML output is separately verified; there is no new notation
admission or new image fixture. Voice removal, lyrics, full Part/stack ownership
and cached extensive head/stem removal remain pending. All global completion
flags stay false.
[Validation](../evidence/musicxml-note-containers/VALIDATION.json).

Use unused output paths with the existing Linux CLI, for example:

```sh
./target/debug/audiveris-musicxml fixtures/official-image-musicxml-cross-system/beam-two-systems.png /tmp/note-containers.musicxml /tmp/note-containers.json
```

The complete `InterCleaner.purgeContainers` is separately pending: pinned Java
`Part.purgeContainers` specifically purges removed lyric lines, not these chord
collections. Lyrics recognition/ownership must be connected before claiming it.

### BeamHeadCleaner retains original contextual grades

Original `org.audiveris.omr.sig.BeamHeadCleaner.process` only removes temporary
BeamHead relations. Native's additional contextualization incorrectly changed
head grades after this ordered LINKS stage. The actual cleaner now removes only
validated edges, keeping vertices, intrinsic/contextual grades and identities.
A later explicitly ordered contextualization still works. One original declared
SIG lifecycle reproduces the old mismatch and verifies the fix/idempotence;
existing image MusicXML regressions are checked separately. This is not complete
post-LINKS raster SIG equality or full cleaner/epilog parity.
[Validation](../evidence/musicxml-beam-head-cleaner/VALIDATION.json).

Source`238c535` passed the complete593-test workspace run (176 binaries+7 empty
doc-test groups), failed0/ignored0, after the rest admission/ownership/containers
and cleaner group. Source was fixed during execution. Independent clone built
the native Linux CLI;20+11 events still match original/prior XML bytes and XSD.
Original full590 at`322135d` remains an earlier source-specific record.
[Full validation](../evidence/musicxml-beam-head-cleaner/FULL-VALIDATION.json),
[readback](../evidence/musicxml-beam-head-cleaner/READBACK.json).

The next post-chord stem-removal observation is original-only preparation:
`fixtures/official-stem-chord-geometry/post-stem-removal.json` shows a HeadChord
remaining a HeadChord without a stem, with refreshed head-only bounds/locations.
Native cached post-chord callback and class propagation are not implemented by
this observation. [Dependency](../evidence/musicxml-beam-head-cleaner/NEXT-DEPENDENCY.md).

### Cache-owned stem removal after CHORDS

Original `ChordStemRelation.removed` now invalidates the surviving HeadChord
inside the retained retriever's staged SIG transaction, before incident edges
are pruned. HeadChord and RestChord class is carried separately from the optional
stem. Pre-rhythm container refresh reads the actual typed ChordStem relation; a
stemless HeadChord keeps its head membership, head-only bounds and note locations
and stays in the measure's head-chord list.

The saved original one-condition post-stem observation matches native before/after
geometry, grades, members and live counts. Native uses an actual image-backed
registry glyph and canonical cache Arc; callback failure is atomic, repeated
removal is idempotent, and the physical registry is retained. This is a declared
lifecycle comparison without HeadStem relations, not automatic raster parity
of that branch. The original fixture's `native_connected:false` records its
earlier preparation checkpoint. Ordinary image/whole-breve guards are retained.
Voices, lyrics, complete Part cleanup and general epilog remain pending.

Existing executable example (use fresh output names):

```sh
cargo build --offline --locked --bin audiveris-musicxml -j1
audiveris-musicxml fixtures/official-image-musicxml-cross-system/beam-two-systems.png out.musicxml out.json
```

[Validation](../evidence/musicxml-post-chord-cache/VALIDATION.json).

### Extensive HeadChord removal preserves cached stems

Original `AbstractInter.remove(true)` now removes an actual HeadChord's
unshared Head members through the existing staged music-removal path. Members
contained by another live chord remain. A stem is linked by ChordStemRelation,
not Containment, so it stays registered in the same retriever/cache/physical
registry. Removed HeadStem relations update only surviving endpoints' abnormal
state, matching the original callback. Frozen-member refusal remains atomic.

Two declared original controls compare exclusive and shared Head membership,
remaining SIG relations/geometry/grades and pre-rhythm staff/measure container
projection. Native's stem grade is independently assessed from an actual image
glyph and preserved; the original control's declared intrinsic0.8 is not claimed
as builder parity. This does not certify raster generation of this deletion
branch, Voice ownership, lyrics or complete cleaner/epilog. Existing image CLI
examples remain available; no new image admission guard is lifted.

[Validation](../evidence/musicxml-head-chord-removal/VALIDATION.json).

### Retained single-voice chord ownership

The admitted ordinary one-staff/single-voice image path now retains Voice objects
with actual chord IDs, starting staff and owning measure. `Voice.addChord`
deduplication/center ordering and original first-staff HIGH voice ID generation
feed event/SIG voice fields and the MusicXML writer; the writer no longer supplies
its own fixed voice label. CLI JSON adds `voices` for inspection.

Original `Measure.removeVoiceChord`/`Voice.removeChord` is consumed by actual
container refresh: last-chord removal deletes that voice, while another chord
keeps the same voice. Two original stored-voice observations extend the existing
head-chord ownership regression; no new image/condition matrix is added. This
cleanup is a declared original/native retained-owner comparison. There is still
no post-rhythm CLI correction endpoint; general polyphonic matching, slots,
merged parts, beam/tie voice propagation, lyrics and complete epilog are pending.

Existing command/input examples above continue to apply. No image admission
guard is removed; existing image MusicXML comparison is distinct from these
controlled ownership operations.

[Validation](../evidence/musicxml-voice-chord-ownership/VALIDATION.json).

Full native workspace validation at source95e191b:595 passed,0 failed/ignored,176 executables and7 empty doc-test groups. A transport disconnect interrupted the initial run after136 executables; only40 remaining executables were resumed from the unchanged Cargo inventory (the interrupted4-test binary was repeated). [Full validation](../evidence/musicxml-voice-chord-ownership/FULL-VALIDATION.json). This is separate from full593 at source238c535.

### BeamGroup chord/Voice getters from the retained graph

The native image report now adds `beam_group_voices`. Original
`AbstractBeamInter.getChords` follows actual BeamStem/ChordStem relations and
BeamRest -> RestInter -> RestChord Containment; `BeamGroupInter.getChords`
deduplicates these lists and sorts by left abscissa, after each beam's center
sort. The group's Voice is the first actual chord's stored owner, including
measure identity (Voice1 can occur separately in multiple measures).

The existing two-system input's two saved original RHYTHMS groups match. A
separate declared original BeamRest control checks rest ensemble/order with an
unassigned rest Voice; this does not claim automatic rest recognition/export.
Canonical Stem/cache/glyph identities remain unchanged. General group/tied Voice
propagation, slots, lyrics and complete cleaner/epilog remain pending.

Try the existing Linux CLI and inspect the new report field:

```sh
audiveris-musicxml fixtures/official-image-musicxml-cross-system/beam-two-systems.png output.musicxml report.json
```

[Validation](../evidence/musicxml-beam-group-voice/VALIDATION.json).
Latest full595 remains the earlier source95e191b; this unit uses targeted
regressions. All global completion flags remain false.

### Chord -> BeamGroup same-measure Voice propagation

The image export now assigns actual chord Voices through original
`AbstractChordInter.setVoice` / `BeamGroupInter.setVoice` behavior in the admitted
ordinary untied chord family, replacing a direct copied field assignment.
A newly assigned chord extends its retained owner to related beamed chords and
interleaved rests in the same actual Measure; Voice IDs repeat across measures
and do not authorize cross-measure propagation. Ownership additions deduplicate
and stably sort by center abscissa. Reassignment to the same owner is a no-op.

Two declared original BeamRest controls (same/other measure on an existing page)
match the added rest's Voice and complete stored chord order. They do not claim
automatic BeamRest recognition, duration or export. Existing successful Linux
CLI inputs/commands above remain valid and now execute this propagation path.
SmallChord/measure-rest/tied branches, multiple-group tail priority, general
slots/voice matching, lyrics and complete cleaner/epilog are still pending with
explicit guards. No admission guard was removed.

[Validation](../evidence/musicxml-beam-group-propagation/VALIDATION.json).
Latest full595 remains earlier source95e191b; this unit uses targeted tests.
All global completion flags remain false.

### Actual first beam-group selection for Voice propagation

Original `AbstractChordInter.getBeams` and `RestChordInter.getBeams` now choose
the actual beam ensemble for the native Voice assignment path. Beam medians
are evaluated at the integer chord center x and stably ordered from the tail
toward the head (ordinary chord head location, or rest-chord center). The first
beam's incoming Containment supplies its actual group, including a chord
connected to more than one group; registration order does not pick the group.

Two declared original competing-group controls, after original chord cache
invalidation, compare selected group/order and same-measure rest Voice. The old
saved original median is209; the native raster median is209.034.., so identical
old median floats are not claimed. This is not full-image competing-group
generation equality. Existing successful CLI commands above remain valid.
Special chord/measure-rest/tie propagation, general rhythm/voice matching,
lyrics and complete cleaner/epilog remain pending. The previous unit's
multiple-group-order guard is replaced by this original selection algorithm;
image admission guards remain. All global completion flags remain false.

[Validation](../evidence/musicxml-chord-beam-order/VALIDATION.json).
Latest full595 remains earlier source95e191b; current targeted results are
recorded separately.

### Following classified tied chords and Voice ownership

Original `AbstractChordInter.getFollowingTiedChords` now follows retained
SlurHeadRelation LEFT/RIGHT heads, the SlurInter's already-classified `is_tie`,
and the right head's actual first chord ensemble. It stably sorts by left
abscissa without silently deduplicating shared head references. Ordinary Voice
assignment visits beam-group recursion before following ties and propagates
only within the same actual Measure; identity equality terminates repeats.
This path is connected to the image CLI's existing Voice assignment.

Four declared original classified-relation controls compare same/other measure,
non-tie and source-on-right behavior. They are not automatic curve recognition,
`SlurInter.checkStaffTie`, full tie rhythm or MusicXML tie export. Current
successful image commands remain valid; image admission has not been expanded.
Special SmallChord/measure-rest, general slots/voices and lyrics remain pending.

The next true epilog dependency is pinned `LinksStep.doEpilog` ->
`SlurInter.checkStaffTie`: actual SlurHead links, same Staff, step/octave and
measure-accidental compatibility, measure-span bounds, head-core/tail lookup box,
mirrors and competing chord/beam-group invasion checks. This needs real CURVES
and full musical head/accidental context. `Part.purgeContainers` separately
removes deleted real LyricLine containers; it needs TEXTS/lyrics/Part ownership.
Neither is replaced by generic deletion or treated as complete here.

[Validation](../evidence/musicxml-voice-tie-dependency/VALIDATION.json).
Latest full595 is earlier source95e191b; this unit has targeted regressions.
All global completion flags remain false.

### Voice's actual last-chord clock -> complete-measure consistency

The image CLI now verifies its accumulated measure time against the retained
Voice's actual last chord, and adds `voices[].end_time` to its JSON report.
Original `Voice.getLastChord` / `AbstractChordInter.getEndTime` gives null when
duration/offset is unknown and otherwise adds the resolved duration to offset.
An original timed-page observation changes1 ->3/4 after last-chord deletion,
then null after the remaining last chord offset is cleared; native actual
Graph/container/Voice callbacks match. This is not underfilled-measure export.

The existing exact 1/16 whole-note tick formatter is shared with the owner clock;
unsupported rational grids and overflow remain guarded. This is not full
Rational/tuplets/negative-time or full `Voice.checkDuration` termination/excess/
measure-stack state. General slots, MeasureFixer/pickup, automatic ties/lyrics
and complete cleaner/epilog remain pending. Existing CLI examples and strict
admission/refusal contracts are preserved; all global completion flags remain
false.

[Validation](../evidence/musicxml-voice-end-time/VALIDATION.json).
Latest full595 remains earlier source95e191b; this unit uses targeted regressions.

### Normal Voice.checkDuration state in the image CLI

Original normal-voice duration checks now update the retained Voice's
`termination`/`excess` and the admitted one-part/one-staff measure's expected
time, abnormal flag and stack-excess maximum. A short voice sets negative
termination only; a long voice sets positive excess and abnormal. Missing
offset marks abnormal without clearing prior termination/excess. Four
sequential original saved-state controls match these histories.

The CLI JSON exposes `voices[].termination`, `voices[].excess`,
`measures[].abnormal` and `measures[].stack_excess` alongside the prior end_time.
Existing image examples/commands are preserved; incomplete/excess measures
continue to refuse export, rather than claiming automatic correction.
Current exact1/16 ticks and the single-part-per-stack admission remain. General
shared stacks/Rational/whole-measure-rest Voice/slots/voice matching/MeasureFixer
and pickup, automatic curves/ties/lyrics/full cleaner/epilog are pending.
All global completion flags remain false.

[Validation](../evidence/musicxml-voice-duration/VALIDATION.json).
Full595 remains earlier source95e191b; this unit uses targeted regressions.


## 保持済みstack slotへの接続（2026-10-06）

原版 `StackRhythm.generateStackSlots → Voice.completeSlotTable` を画像CLIの解決済みChord時刻へ接続。実Chord IDのslot/BEGIN/CONTINUE表を保持します。保存原版ページと、別Voiceの宣言時刻による継続・終端同時刻・欠落を比較。一般共有stack・幾何slot/voice matching・全小節休符Voice等は未完で、複声部の画像自動処理の完成ではありません。手順と境界は [MUSICXML-LIMITED.md](MUSICXML-LIMITED.md#solved-chord-clocks--retained-stack-slots-2026-10-06)、証跡は [VALIDATION](../evidence/musicxml-stack-slots/VALIDATION.json)。全595は95e191bでの既存結果です。


## slot由来のVoice・stack duration（2026-10-06）

原版 `Voice.getDuration / MeasureStack.getSlotsDuration / StackRhythm.readStackActualDuration` を保持済みslotと画像CLIへ接続。BEGIN終端の最大値・不明slot時刻・slotなし・Voiceなしを原版5状態と比較し、CLIのVoice durationと保持stack実durationへ反映。全小節休符Voiceの構築、一般共有stack・voice matching等は未完。詳細は [CLI手順](MUSICXML-LIMITED.md#slot-based-voicestack-duration-2026-10-06)、[検証](../evidence/musicxml-slot-duration/VALIDATION.json)。全595は95e191bの結果です。


## 実Chordの時刻伝播（2026-10-06）

原版 `setAndPushTime` の明示Voice順序→beam group→判定済みtieを、実SIG・正確な所有小節で接続。画像CLIのslot生成前に実行し、伝播後の時刻と既存イベント時計の整合を検査します。原版5条件を比較。一般slot/voice matching・共有multi-part stack・全小節休符Voice・自動slur/tie・lyrics等は未完で、完成フラグfalse。 [手順・境界](MUSICXML-LIMITED.md#live-chord-time-propagation-before-slots-2026-10-06) / [検証](../evidence/musicxml-chord-time/VALIDATION.json)。全595は95e191bの既存結果です。


## 全小節休符Voiceの画像縦断（2026-10-06）

原版で正しく通る1画像（四分4音＋全小節休符）をnativeでも出力。認識済みWHOLE_RESTから専用Voice・slotなしstack音価・MusicXMLのmeasure休符/type省略へ接続。Chord/Voiceのintrinsic duration/end/terminationはnull、通常Chord列は空の原版契約を保持。原版同一画像の5イベント・bounds・Voice/stack状態と、別枠のprocessing-switch形分類8条件を比較。全595は95e191bの既存結果、全体完了false。 [試せる入力と実行例・未対応](MUSICXML-LIMITED.md#default-whole-measure-rest--musicxml-2026-10-06) / [検証](../evidence/musicxml-measure-rest/VALIDATION.json)。一般共有stack/slot-voice matching、lyrics、自動slur/tie、完全cleaner/epilog等は次の依存段階として残します。


slot位置・最近傍参照（2026-10-06）：原版Slot.computeXOffsetの実chord中心平均、
stack相対X、Math.rintを単staff/zero-skewの実所有slotへ接続。
MeasureStack.getClosestSlotの同距離時先行slot保持/空slotを保存済み原版で比較。
既存2system20slot、全小節休符の空slot、CLIのread-only --slot-atを確認。
共有multi-staff補間/一般point-to-staff選択/SlotsRetriever・ChordsMapperによる
一般voice matching/補正操作は未完。今回対象3件・fmt/clippy、全595は95e191b。
[検証範囲](../evidence/musicxml-slot-geometry/VALIDATION.json)。


Measure.resetRhythm/AbstractChordInter.resetTiming（2026-10-06）：
画像CLIのrhythm前に対象小節のVoice・異常フラグ・実chordのslot/voice/timeを
解除。原版が保持するintrinsic音価・stack slots/expected/actualは保持し、
他小節・relations・registry/cacheを維持。保存済み原版normal6/rest1chordの
宣言reset/repeatを比較。一般slot retrieval/matching・SmallChord・共有stack・
補正再処理/persistenceは未完。対象2件・fmt/clippy、全595は95e191b。
[検証範囲](../evidence/musicxml-rhythm-reset/VALIDATION.json)。


getStemDirとMusicXML stem（2026-10-06）：原版SlotsRetriever隣接判定の前提
となる実HeadChordのhead/tail座標差を接続し、PartwiseBuilderの通常四分/二分を
含むstem出力にも伝播。現在one-head制限下の座標比較を使い、flag形による方向
推定を置換。beamの実HeadStem identity guardは維持。原版保存済み24chord
getter/独立4宣言条件、既存MusicXML16＋getter/Voice2対象検証、その後追加
アサート2だけ再確認。全595は95e191b。通常音符stemの追加以外の既存XMLは
再取得時に比較し、旧bytes/hashは元ソースの証跡として保持。一般slot/voice・
複合特殊headのnote別engravingは未完。
[検証範囲](../evidence/musicxml-chord-stem-direction/VALIDATION.json)。


全休符ページの境界（2026-10-06）：小さい1入力を原版full batchへ一度だけ
通したが、原版MusicXML divisions=0でXSD拒否。native既存コードはhead0の
2全小節休符・各4拍・XSD有効だが、原版MusicXML成功一致として数えない。
この比較の再実行/座標探索/独自の原版音価捏造は行わない。
[観測と原本](../evidence/musicxml-all-measure-rest/ORIGINAL-LIMIT.json)、
[次の原版依存](../evidence/musicxml-all-measure-rest/NEXT-DEPENDENCY.md)。

原版SlotsRetrieverの前段slot取得（2026-10-06）：明示分離→同時刻→beam→
mirror→root stem→中間同時刻→隣接→近接→遠隔の優先順と全メンバー互換条件を
実所有画像chordへ接続。narrow/wide候補はJSONのpreliminary_measure_slotsで参照。
保存ページ24 chord/6小節と宣言5条件の比較は、画像MusicXML回帰と区別します。
複数chord slotの一般Voice割当、compound slots、共有stack、全Rationalなどは
未完。既存画像受付範囲は拡大せず、全体フラグfalse/最終非UI省略なしを維持。
[CLI例と境界](MUSICXML-LIMITED.md#preliminary-narrowwide-measureslots-2026-10-06)。

compound slots（2026-10-06）：原版narrow/wide交差区分、縦位置順、
getNarrowSlotを画像CLI候補列へ接続。保存済み6小節と宣言4境界を比較。
原版cursorの重なり挙動は宣言条件として保持し、成功画像とは扱いません。
一般Voice matching/候補slot setter時間同一性/共有stackは未完。
[CLI参照と未完](MUSICXML-LIMITED.md#compound-slot-membership-2026-10-06)。

先頭Voice/ZERO伝播（2026-10-06）：実リンク由来intrinsic音価を先に解決し、
全chordの先行Voice登録をやめ、first compound/全小節休符の先頭chordから
原版setVoice→beam/tie→setAndPushTime(0)へ接続。保存成功2ページ6小節の
途中Voice/時刻/音価を比較し、後続の一般rookie/active matchingとは区別。
[試し方・現在の境界](MUSICXML-LIMITED.md#starting-voice-and-original-zero-time-propagation-2026-10-06)。


## 並行Voiceの音価接続（2026-10-06）

原版5.11が同一画像から正常exportした通常Bravura・五線・ト音記号・C major・
4/4・単part/staffの二声部で、既存四分/遅延Voiceに、八分連桁18音・二段十六分19音・
旗付き八分17音・二分14音を接続しました。原版のshape音価とbeam+flag短縮、
実group/Voice/slot所有・時刻、全Voice音価検査からnative MusicXMLへ伝播します。
9入力135イベント、以前の67イベントのXMLバイトを維持。宣言条件のmapper比較、
保存状態getter比較、同一入力の原版全画像認識/exportは区別します。
[CLI例と境界](MUSICXML-LIMITED.md#parallel-half-note-voice-from-an-actual-image-2026-10-06)、
[今回の対象検証](../evidence/musicxml-parallel-half/VALIDATION.json)。

これは処理途中の通常Voice接続で、同一glyphの自動parallel stem aliasの既知拒否を
外したものではありません。mixed rests・他並行拍子・共有/Merged stack・一般連符/
Rational・複数head chord・lyrics/slur/tie・完全cleaner/補正/export等は未完です。
非zero skew・別font・実スキャン・特殊staff等の未検証も保持。非UI全機能を省略しない
目標と完了フラグfalseを維持し、全595件は旧95e191bの結果として扱います。

## Same-stem multi-head note/export continuation (2026-10-06)

Original `AbstractChordInter.getHighestNote/getMembers`, `HeadChordInter.getHeadsBounds`,
`VoiceDistance.Separated` and `PartwiseBuilder.processChord/processNote` now feed
the actual image-owned ordinary multi-head path. All heads retain their
Containment owner, Voice/onset/duration. Notes export bottom-up; only subsequent
notes get `<chord/>`, and the ensemble advances the Voice once. One original
full-image two-measure comparison contains12 notes/eight chords; saved recognized
member/Voice/slot/getter comparisons are separate from that image export.
Mixed mirror/special heads, shared stacks, general tuplets/rests and full
cleaner/epilog remain partial; no original nonUI feature is omitted.
[Validation](../evidence/musicxml-chord-members/VALIDATION.json).

Multi-head beam continuation: original14-note image validates four actual
eighth-chord clock advances and per-head begin/continue/continue/end output.
Live member order supplies both `events[].chord` and MusicXML indications;
no Voice/timing is rewritten by that binder. Original saved group/shortening
getters are distinct from full-image export.
[Validation](../evidence/musicxml-chord-beam/VALIDATION.json).

FLAG2 continuation connects original flag values, graded multi-source ordinary
Stem direction and FlagStem registration to exact sixteenth durations. One
original full-image15-note comparison is distinct from a read-only actual SIG
lookup/direction control. Higher values, small/percussion/mixed head branches
and general nonUI parity remain partial.
[Validation](../evidence/musicxml-chord-flag2/VALIDATION.json).

## True first-dot continuation (2026-10-06)

`DotFactory` late note lookup, owned augmentation, chord dot counting and
dotted intrinsic duration now feed ordinary native Voice/Partwise output.
One original-successful eight-event image matches musical contents including
dots; saved getters and declared competing CAESURA weak-purge control are
separate evidence. Double dots and shared/rest/repeat/staccato dot branches,
positive pause notation, tuplets/shared stacks/special heads/lyrics/slur/tie
and full cleaner/epilog remain partial, not omitted.
[CLI example](MUSICXML-LIMITED.md#true-first-augmentation-dot-2026-10-06),
[validation](../evidence/musicxml-augmentation/VALIDATION.json).

Double-dot continuation connects original phase2 glyph/DoubleDot ownership,
second support and exact7/4 intrinsic duration to native Voice/MusicXML.
Original integer point and geometric-impact grading are shared with the
pause path. One eight-event original full image is distinct from actual saved
getters and declared weak pause/unlinked flat/zero-weight gap controls.
First/second same-glyph alternatives, positive accidental semantics and full
cleaner/epilog remain pending. [Validation](../evidence/musicxml-double-augmentation/VALIDATION.json).

Original inconsistent multi-head `countDots` now leaves an actual discarded-dot
receipt for image admission. One original-successful12-note input exports
undotted chords; declared-grade original automatic/manual controls are
separate. Manual dots stay, frozen loss refuses atomically, and unknown missing
dots still refuse. Full nonUI parity/cleaner flags remain false. Next:
`evidence/musicxml-inconsistent-augmentation/NEXT-DEPENDENCY.md`.

The follow-up dotted-rest image is a negative/unconfirmed boundary: original
exports no dot and a7/8 pickup, while native refuses at an earlier foundation
removal callback. It is not a positive feature or native-only rest bug. One
comparison was stopped; no coordinate search or admission expansion. Evidence:
`evidence/musicxml-rest-augmentation-limit/ORIGINAL-LIMIT.json`.

Actual failing negative-input vertex: bare GRID BarlineInter ID2, staff1/system1,
bounds[528,219,2,47]. Original actual GRID lifecycle confirms Staff/SIG removal,
idempotence and glyph retention. The corresponding guarded native removal now
feeds live Staff membership into MEASURES. GRID glyph IDs are not HEADS/STEMS
registry IDs. This is a lifecycle unit, not positive dotted-rest recognition.

StemBuilder.lookupChunks now assembles physical chunk glyphs without forcing
StraightFilament endpoint getters, as in original toGlyph/registerOriginal.
A declared14-pixel square from the earlier failing ink passes original glyph
conversion but throws on a forced endpoint getter. Native keeps endpoint guards
for geometry consumers. The distinct original-successful half-rest image now
reaches the pending rest-support branch; no native dotted-rest success yet.

Ordinary unshared rest augmentation is now connected from DotFactory's actual
rest lookup to owned AugmentationRelation, rest contextualization/weak purge,
RestChord dot count/duration, native Voice and Partwise dot output. The separate
original-successful half-rest image supplies4 matching events and3/4 rest
intrinsic duration. Declared original target-grade controls verify the zero
target support coefficient separately. Multiple/competing/shared support and
orphan-dot cleanup still require original lifecycle; the earlier undotted
negative quarter-rest input remains refused. Full nonUI parity flags stayfalse.

Augmentation/DoubleDot relation removal now refreshes the surviving source
Inter's abnormal state. Original checkAugmentationDots target selection runs
before owned dot contextualization/counting, so actual weak-rest loss no longer
leaves an unprocessable strong orphan. This is verified by temporary original
rest-grade control on saved recognized Inter, not a new weak-rest full-image
success. Physical registry persists; frozen ownership loss and missing rhythm
still refuse. Multiple support selection and complete cleaner/epilog remain
partial; all nonUI functions stay in the final scope.

Rest/DoubleDot competition now uses original strictly greatest positive grade
and relation iteration order (ties preserve first; all-zero removes all).
Actual edge removal refreshes the source dot; incoming rest augmentation also
recounts the actual single RestChord immediately, while DoubleDot removal does
not recount the note chord. The connected dot checks/context/count repeat keeps
owned deletion receipts. Five original declared controls are distinct from
full-image recognition; no new image scope is claimed. Competing-head
upper/lower/integer-pitch selection, multi-member callbacks, post-rhythm editing
and the complete LINK adapter/cleaner/epilog remain partial. Creation/admission
guards and all global completion flags stay in place.

Actual double-dot chain detachment is verified separately from image export:
removing its real head support immediately recounts the single ordinary owned
HeadChord; the next target check removes both dots in order and retains glyphs.
Post-Voice mutation still requires rhythm reset. Original lazy Part getter
caches change whether the removed second dot remains in its Measure collection;
that cached ownership is observed, not yet matched by native. Competing-head
selection, multi-member callbacks and complete cleaner/epilog remain pending.
Evidence: `evidence/musicxml-dot-chain-detach/VALIDATION.json`.

Head support competition now preserves original owned-chord/top-down member
order, stable upper/lower-dot preference and integer-pitch parity of every
supported head (including ties-even rounding). Six declared original controls
verify it independently from image recognition; stronger relation grade is not
the head criterion. Existing CLI examples remain, while multiple/shared support
creation/admission, multi-member callbacks, direct vertex-removal ownership and
complete LINK/cleaner/epilog remain partial. Evidence:
`evidence/musicxml-head-dot-competition/VALIDATION.json`.

Dot vertex removal now recounts surviving actual ordinary note owners. The
same callback handles multi-head mean counts and reentrant inconsistent automatic
dot deletion; manual dots stay. Original actual first-dot chain mutation and
two declared automatic/manual two-member controls are separate from full-image
success. Existing CLI examples/admission guards remain. Part getter/Measure
cached ownership, shared mixed-owner intermediate ordering, discarded-candidate
closure receipts, post-rhythm edits and complete cleaner/epilog remain partial.
Evidence: `evidence/musicxml-dot-vertex-lifecycle/VALIDATION.json`.

Original mixed-owner dot callback order now matches observed intermediate states:
a deleting dot remains SIG-registered until its copied incoming/outgoing edges
have invoked synchronous callbacks; nested automatic deletion occurs before
later owner callbacks. Head-first/rest-first/manual controls and one actual
double-dot chain compare all callback entry/return states. These controls do not
admit shared-support images. Part/Measure caches and LINK candidate closure for
all nested deletions remain partial. Evidence:
`evidence/musicxml-dot-callback-order/VALIDATION.json`.

LINK dot candidate receipts now retain every actual reentrant vertex deletion,
once in unregister order, with original owner/member-count proof only for
inconsistent automatic countDots cleanup. Weak/orphan deletion alone does not
permit a previously unhandled dot image. An original weak-dot control confirms
that its callback also deletes a strong automatic dot; contextualization now
precedes weak deletion to prevent stale snapshot resurrection. Mixed-owner and
weak-cascade controls are separate from the existing five full-image CLI
examples. Part/Measure cached membership and shared-support image admission
remain unconnected. Evidence:
`evidence/musicxml-dot-removal-receipts/VALIDATION.json`.

Ordinary dot ownership now preserves lazy Part caching and actual Measure
membership across SIG lifecycle callbacks. First-dot registration precedes
staff/support assignment; second dots inherit Part through DoubleDot without a
staff. The image CLI populates the dot subset of PageRhythm FRAT ownership before
rhythm reset. Removal updates its containing Measure before SIG edge callbacks.
Actual saved cold/warm controls and original same-glyph factory-order replay
compare these states separately from existing image recognition/export. Full
multi-staff/cross-Part stacks, other FRAT classes, shared-support image admission
and complete cleaner/epilog remain partial. CLI reports expose the retained
state at `shared_sig.augmentation_dot_ownership`. Evidence:
`evidence/musicxml-dot-part-ownership/VALIDATION.json`.

Same retained-dot re-registration now preserves its contextual grade when the
intrinsic grade is unchanged, matching original same-Inter add/remove. Saved
cold/warm controls verify Part persistence and ordered Measure membership through
second-before-first re-addition; original scalar stack boundaries verify
left-exclusive/right-inclusive selection. These controls do not broaden image
admission or demonstrate full correction/editing lifecycle. Existing image CLI
examples and the negative quarter-rest boundary remain the regression scope.
Evidence: `evidence/musicxml-dot-owner-readd/VALIDATION.json`.

Ordinary Head/Rest/dot owner edits now follow original AbstractInter setters:
explicit Part assignment/reset, staff, intrinsic grade and bounds preserve
unrelated cached Part/context and actual Measure membership. DotFactory's image
path uses the staff setter after SIG registration and before support. Actual
saved-chain setter stages and callback entry/return states are a control, not
new image admission. Moving bounds alone does not perform a complete correction
task or repair old Measure membership. Multi-staff/shared support image admission,
general edit tasks, support re-add ordering and complete cleaner/epilog remain
partial. Evidence: `evidence/musicxml-dot-owner-edit/VALIDATION.json`.

Retained support remove/re-add now keeps stable relation identity separately
from live insertion order. Original first-supported Part selection (including
null without fallback), incoming-before-outgoing callback copy and every owner
count/cache/Measure callback stage match one declared shared-support control on
actual saved HeadChord owners. Already-live support replay does not reorder.
Existing image CLI admission is unchanged; this is not a shared-support image
success. Manual support-added callbacks, general edit tasks, multi-staff and
complete cleaner/epilog remain partial. Evidence:
`evidence/musicxml-dot-support-readd/VALIDATION.json`.

Ordinary newly-added Augmentation/DoubleDot support now completes its original
synchronous callback before another relation is inserted. Augmentation checks
source abnormal then recounts its actual chord if dot or relation is manual;
DoubleDot checks its source only. Three original declared controls distinguish
automatic cache retention, manual-dot count1 and manual-relation nested automatic
dot deletion. Callback entry/return compares Part/Measure, grade/context/bounds,
manual state and owner counts; actual nested deletion reaches LINK receipts once.
This is not automatic shared-support image recognition or a complete correction
command. Existing five-image CLI examples and quarter-rest refusal remain;
Part/Measure editing tasks, multi-staff and full cleaner/epilog remain partial.
Evidence: `evidence/musicxml-dot-support-added/VALIDATION.json`.

Ordinary retained-dot DefaultEditor/EditingTask now preserves original do/undo/redo
ordering and relation-object identity. Failed duplicate links are pruned before
undo; new successful links precede old unlink. Editor do detaches glyph and undo
restores bounds/glyph and clears lookup attachments. Part/Measure caches and
membership are not reconstructed. Fifteen original callback/publication states
compare with actual image-owned native Graph/physical registry under explicitly
installed saved cold owner control state. This is not automatic correction or
controller reprocessing, general editor/ensemble/history/multi-staff completion.
Evidence: `evidence/musicxml-editing-task/VALIDATION.json`. Existing image CLI and
negative quarter-rest admission remain unchanged; latest full595 is historical
95e191b, not a current full rerun.

Retained editing sequences now keep original UITaskList forward do/redo and reverse
undo publication order. Original CtrlTask class-based selection starts dot edits
at RHYTHMS; latest HEADS yields no step call, latest PAGE schedules RHYTHMS then
PAGE. Native notifications distinguish DO-only modified intent and planned steps
from actual reprocessing: `reprocessed_steps` is empty and
`controller_impact_complete` stays false. Actual original below-first epilog and
saved two-task order are controls, not automatic CLI correction. Native SheetStub
persistence, preEdit/history/step execution/general editors remain pending.
Evidence: `evidence/musicxml-editing-impact/VALIDATION.json`.

RHYTHMS impact notifications now select actual retained ordinary stacks by edited
Inter center, with original encounter-order deduplication. A saved raw cross-stack
task compares do/redo2,1 and undo1 against original SystemInfo.getStackAt and
RhythmsStep.Impact.add, while cached Part/staff and original Measure membership
remain unchanged. It does not relocate ownership, run StackRhythm/voice refinement
or admit new images. The next handoff is actual stack reprocessing and PageStep:
current image rhythm lives inside retained-SYMBOLS export and cannot consume an
arbitrary edited Graph. Its owner/timed-chord guards remain in place. Full correction,
preEdit/history/general editors/multi-staff/cleaner remain partial. Evidence:
`evidence/musicxml-editing-stack/VALIDATION.json`.

Publication control is queue-flushed original InterService delivery versus native
post-callback publication request records. This does not prove GUI delivery or
async ordering relative to controller epilog. Actual impact execution remains
pending separately from task/owner controls.

Stack rhythm initialization now follows original StackRhythm.process ordering:
expectedDuration is installed before stack-only reset (actual/excess/slots), then
Measure reset and slot retrieval run on the owned live Graph. Stack reset preserves
old Voices, Chord timing/cache and membership. An actual saved original bounds-only
PageRhythm.reprocessStack succeeds; separate original-method stages equal its final
state. Native corrected-owner entry and system-wide Voice refinement remain pending;
this checkpoint integrates the reset/order into the existing image export only.
Evidence: `evidence/musicxml-stack-reprocess/VALIDATION.json`. Full595 remains
historical95e191b; no new full rerun or broader image admission.

A separate retained-export correction entry now runs bounded actual stack rhythm:
`Export.apply_editing_sequence` consumes typed same-stack ordinary-dot bounds
commands, preserving cached expectedDuration and Part/Measure membership. It does
not falsify the SYMBOLS report or call populateFRATs/LINKS. FinalCheck now precedes
Stack slots/actual as in original. Actual HIGH Voice purge/sort/rename follows;
linked tie/beam/NextInVoice/preferred-ID refinement is explicitly pending. Native
writer consumes the resulting owner; failure rolls it back. `--edit-dot` makes
this bounds-only raw-task path runnable after image recognition. This is manual
correction execution, not automatic correction discovery or full controller/Page
processing. Original full saved-owner processing and fresh-HIGH cleanup controls
are separate from existing five-image47-event automatic image regression.
Evidence: `evidence/musicxml-owned-reprocess/VALIDATION.json`.

Actual adjacent-Measure NextInVoice and first-Chord preferred-ID Voice refinement
now follow original order after HIGH purge/sort/rename. Collision swaps update
real retained Voice objects and their owned Chord projection, then output binds
those owners; clocks/slots/cache/membership stay unchanged. Original declared
parallel-owner comparison gives2,1 after NextInVoice, then1,2 after preferred1.
This is not automatic link/ID-preference discovery or new image admission. Tie,
cross-Measure beam refinement, timed support/PageStep and general correction
remain pending. Evidence: `evidence/musicxml-voice-refinement-links/VALIDATION.json`.

Original retained HIGH refinement now resolves incoming classified ties, then the
first Chord's actual BeamGroup containing the preceding Measure and its immediate
group predecessor, before NextInVoice and preferred ID. Real-owner controlled
IDs2/1 ->1/2 ->2/1 ->1/2 match original; membership/clocks/slots remain unchanged.
This supersedes the earlier tie/beam refinement boundary only for retained-owner
processing, not automatic link discovery, cross-bar image admission or tie export.
Evidence: `evidence/musicxml-voice-tie-beam/VALIDATION.json`. Timed support editing,
general Voice kinds/PageStep/shared stacks/full cleaner remain explicitly partial.

Original partial-rhythm null reference control revealed and fixed a native
rename mismatch: retained Voice.chords does not grant Chord.voice. Renaming and
ID collisions preserve null instead of recreating it, so incoming ties skip that
source and try the next live relation. Actual membership/clocks/slots/cache are
unchanged; this control does not admit a new null-reference image/export.
Evidence: `evidence/musicxml-voice-null-reference/VALIDATION.json`. Next BeamGroup/
NextInVoice nullable predecessor boundaries and general owner/controller remain.

Current Chord.getVoice is now resolved on actual retained Part/Measure owners
after prior refinement swaps. Tie skips null, but original BeamGroup/NextInVoice
dereference null and fail; native returns an explicit guarded error with staging
rollback. Two original exceptions are failure evidence, not MusicXML success.
Existing positive ordered swaps/manual-dot reprocessing are retained. Evidence:
`evidence/musicxml-voice-reference-guards/VALIDATION.json`. First-Chord-null/general
Voice kinds and timed callback duration lifecycle remain separate dependencies.

Retained timed HIGH Voice.getFirstChord also remains valid when its Chord Voice
reference is null. Original incoming tie gives2/1, preserving the null first
Chord/Head reference; native matches actual membership without assigning it.
Event binding continues to reject null, so this is owner-control success only,
not automatic image/export acceptance. Existing Part/Measure/staff/slot and
conflicting-reference guards remain. Evidence:
`evidence/musicxml-voice-first-null/VALIDATION.json`. Timed support callback/live
duration refresh and general Voice/shared/controller remain next dependencies.

### Timed ordinary dot callback checkpoint

Original `AugmentationRelation.added/removed` and uncached
`AbstractChordInter.getDurationSansTuplet/getEndTime` are connected for retained
ordinary timed chords. `countDots` changes live duration without clearing clocks,
Voice or Part/Measure membership. A saved real double-dot owner control observes
link-add before unlink-remove; manual and automatic re-add behave differently.
This does not admit automatic shared-support images or timed correction export;
that guard awaits task/reprocess/rollback comparison. See
`evidence/musicxml-timed-dot-support/NEXT-DEPENDENCY.md`.

### Retained support transfer / real reprocessing checkpoint

`--move-dot-support` now exposes the compared ordinary single-head same-stack
augmentation transfer as an explicit correction after image recognition.
EditingTask callbacks run with existing clocks/owners; actual stack reprocessing
retains expectedDuration and does not call populateFRATs/LINKS. MusicXML music
matches original PartwiseBuilder for the declared transfer. Manual subsequent
transfer supports do/undo/redo. Original automatic re-add instead yields13/16;
native keeps the complete-measure refusal and rolls back all retained state.
This is not automatic correction discovery or full controller/history parity.
`evidence/musicxml-timed-support-reprocess/NEXT-DEPENDENCY.md` lists the remainder.

### Retained Book/SheetStub callback state

Actual retained single-image editing ownership now records ordinary dot abnormal
callback dirty propagation separately from controller DO requests and publication.
Original Book/SheetStub setters, do/undo/redo intermediate flags, unchanged-value
and below-first-step boundaries match the saved actual owner control. Binding
begins after initial recognition/rhythm: this does not reconstruct persisted or
initial-processing flags. MusicXML render does not clear dirty or complete PAGE.
Other SIG dirty sources, score reduction, repository/multi-sheet/project persistence
remain partial. Evidence: `evidence/musicxml-editing-book-dirty/VALIDATION.json`.
Latest targeted1+3 and fmt/clippy passed; full595 remains historical95e191b.

Retained HeadStemsCleaner relation deletion now invokes abnormal updates per
removed edge in supplied order, Head before Stem, and marks the actual editing
Book/SheetStub only for changed values. A saved original owner with declared
alternate support compares callback boundaries without claiming automatic image
recognition or stem glyph geometry parity. Added/manual HeadChord mutation,
generic vertex lifecycle and complete dirty coverage remain partial. Evidence:
`evidence/musicxml-head-stem-dirty/VALIDATION.json`.

Retained automatic HeadStem re-add now compares per-edge abnormal/dirty callbacks
and same-object duplicate behavior. This does not change the pre-recognition
batch contract or admit manual HeadStem editing. Original manual controls expose
nested ChordStem removal (temporarily null stem) then addition/cache invalidation,
while dirty can stay false. Actual retained Chord/cache/rhythm integration is the
next dependency, guarded until compared. Evidence:
`evidence/musicxml-head-stem-added/VALIDATION.json` and `NEXT-DEPENDENCY.md`.

### Retained HeadChord lazy cache

Existing OwnedMusic Chord identities now bind actual lazy bounds/head/tail/beams
cache. Per-edge ChordStem callbacks invalidate it without clearing Part/Measure/
Voice/slot/time; getters recompute from live members/current stem and snapshots
remain read-only. Legacy geometric metadata is a derived projection, not cache
state. Native manual HeadStem admission is still guarded. Original actual manual
reprocess creates parallel Voices and7/64 on this flagged target, which remains
outside native fine-duration admission. This is owned control comparison, not
automatic image success. Evidence: `evidence/musicxml-manual-chord-cache/VALIDATION.json`.

### Retained manual HeadStem owner callback

The explicit ordinary retained correction stage now follows HeadStem.added:
nested old ChordStem removal, new ChordStem addition/cache invalidation, then
Head/Stem abnormal checks. The same OwnedMusic Chord receives live getter
stem/geometry; members, Measure, Voices/slots/time and canonical Stem Arc stay
intact. The original listener’s cold/warm intermediate reads are compared.
New ChordStem is not promoted to manual. Generic manual and export provenance
guards remain; this is not a new automatic image/CLI admission. Exact7/64 and
underfilled Voice reprocess/export are next dependencies, not original failure.
Evidence: `evidence/musicxml-manual-chord-owner/VALIDATION.json`.

### Exact ordinary duration/endTime getters

Ordinary retained getters now follow original reduced Rational dot factors and
timeOffset addition, including7/64 at all six manual nested callback boundaries.
This uses current live relations, not stored duration metadata. The separate
sixteenth-grid adapter and EditingTask admission still refuse that fine duration;
Rational clocks/Voice processing/underfilled export and manual CLI transactions
remain pending. This does not expand automatic image acceptance or complete
signed/general Rational parity. Evidence:
`evidence/musicxml-manual-chord-duration/VALIDATION.json`.

### Changed Chord reprocess refusal and rollback

The existing edit checkpoint now retains actual Chords, including stem/bounds/
members. A control invokes the real selected native rhythm pass after the owned
manual callback: retained expectedDuration survives, clocks reset only in the
legitimate rhythm pass, and native fails at7/64 rather than stale ownership.
Rollback restores actual Graph/cache/Measure/Chord/Voice and original XML/
provenance. This is native refused processing, not manual export/automatic image
success; original Rational reprocess still succeeds. Evidence:
`evidence/musicxml-manual-chord-reprocess/VALIDATION.json`.

### Fresh ChordStem identity after owner return

Returning a declared manual HeadStem to a previously removed Stem partner now
creates a fresh ChordStem registration, as original HeadChord.setStem does; the
old relation stays removed and an already live partner is kept. A real saved
owner control detected and fixed the earlier native ID replay, with six return
callback boundaries and dynamic7/64 ->7/16 duration/endTime compared. Manual
Support ratios derive from supplied grade/consistency; new ChordStem is not
promoted. Part/Measure/member/Voice/slot/time are retained. This is two forward
additions, not general task undo or new native export/automatic acceptance.
Evidence: `evidence/musicxml-manual-chord-identity/VALIDATION.json`.

### Returned-owner rhythm and native MusicXML control

Ordinary correction CLI and retained owner controls now share the actual selected
stack/refine/render helper. Two declared manual additions returning to the original
Stem produce eight native notes with original expected/actual timing, Voice/slot/
chord states and musical contents; XML passes XSD and equals prior native bytes.
Retained getBeams cache is read during live duration resolution. First7/64 remains
a refused/rolled-back control; generic/manual provenance guards stay. This is not
new automatic recognition or general task/CLI admission. Evidence and native XML:
`evidence/musicxml-manual-return-reprocess/VALIDATION.json`, `control.musicxml`,
`TRY.md`. Full595 remains historical95e191b.

### Rational actual Chord clock propagation

Original sequence/group/tie clock propagation now uses exact ordinary fractions
and live getEndTime instead of projected cached duration. Real saved-owner
manual7/64 plus declared NextInVoice controls compare0,1/64,null and cross-Measure
boundaries, retaining owner/expectedDuration/dirty state. Slot/Voice Rational
clocks, underfilled export and actual task publication remain pending; the
image mapper/public correction guards stay. See
`evidence/musicxml-rational-chord-clocks/NEXT-DEPENDENCY.md`.

### Rational stack Slots, Voice termination and owned export

Actual expected/actual/excess, Slots and Voice duration/end/termination now use
exact fractions. Native owned algorithms with a declared original Voice plan
match the real original manual7/64 reprocess: expected1, actual9/16, two short
Voices9/16 and7/64, termination-7/16 and-57/64,8notes/divisions16/backup36.
Divisions inspect all owned page Chords, not only a selected Measure. No ending
forward/padding is added. This is controlled scheduling, not automatic mapper
or public fine correction admission; those guards and13/16 refusal stay. See
`evidence/musicxml-rational-voice-clocks/TRY.md` and `NEXT-DEPENDENCY.md`.

### Actual Rational candidate mapper and Voice extinction

Native narrow/wide/compound retrieval, starting Chord construction, rookie
mapping/retry and extinction now choose the manual7/64 control Voice plan
without supplied assignments or output times. Actual clocks use exact fractions
and live duration/endTime, producing original-matching2 underfilled Voices and
8note native output, identical to the earlier declared-plan bytes/XSD. The
selected-stack duration/event adapter and task/controller publication still
guard public correction acceptance. See `evidence/musicxml-rational-mapper/TRY.md`.

### Exact selected-stack owned correction processing

Common selected analyze/refine/rebind/render now resolves live ordinary fractions
through actual Slot/Voice clocks and native export. The original saved-owner
manual7/64 control matches two short Voices and8notes/divisions16/backup36 in a
test-only observed-owned scope. Public image/correction guards remain; actual
RelationTask/controller comparison is still required, and automatic13/16 stays
incomplete. See `evidence/musicxml-rational-selected-reprocess/TRY.md` and
`NEXT-DEPENDENCY.md`. Target module13 passed; full595 is historical95e191b.

### Actual retained HeadStem LinkTask

Original real LinkTask/UITaskList and CtrlTask RHYTHMS comparison now covers
DO/UNDO/REDO callback/cache/owner clocks6/2/2, publication1/0/1 and manual status.
Undo removes the fresh HeadStem without restoring old ChordStem; exact7/64 and
short Voices remain through native selected reprocessing/export. Tasks bind to
actual canonical Stem/Chord ownership; task failures restore typed Chords/Graph.
Notification is planned impact, not PAGE/project completion. See
`evidence/musicxml-headstem-tasks/TRY.md` and `NEXT-DEPENDENCY.md`. General
UnlinkTask/preLink/controller and public fine correction admission remain pending.

### Actual retained HeadStem UnlinkTask

The same canonical owner now performs actual UnlinkTask do/undo/redo after
LinkTask. Original callback/cache/clock order2/2/2 and publication0/1/0 match;
undo re-adds the same HeadStem while retaining new ChordStem/7/64. Exact selected
reprocessing yields the same original8note short-Voice stream in all6 operations.
`evidence/musicxml-headstem-unlink-tasks/TRY.md` reproduces the controlled stage.
Public admission still needs preLink Addition/Removal/Containment member lifecycle;
no new automatic image or HeadStem CLI success is claimed.

### Retained existing-target preLink and empty Chord lifecycle

Original `HeadStemRelation.preLink` existing-target task order now runs through
real Containment unlink, empty-Chord RemovalTask and member transfer, followed by
the declared raw manual LinkTask. Twelve task boundaries and thirty listener
entry/return observations agree under the declared saved numeric controls.
Removal retains actual identity/Part/Measure/Voice/Slot clocks, removes Measure
and Voice-list membership, and undo re-registers the same relations without
restoring the Voice list. Rollback now includes typed Chords, Measures and Voices.

The comparison rounds saved grades and declares original Stem bounds on the same
canonical native Stems. Existing native image Stem extent is five pixels longer
into the source head; this automatic-image discrepancy remains unresolved.
New-target AdditionTask/unplaced Part/Measure is now compared at the next
checkpoint below. Canonical mirrors, controller conflicting links and legitimate
reprocess/export admission remain dependencies.
All nonUI functions remain in scope and completion flags remain false.
See `evidence/musicxml-headstem-prelink-existing/TRY.md` and `NEXT-DEPENDENCY.md`.

Cloud handoff evidence is distinct: `189f214` and tree
`55755c176e8d32617efb1e90590002073a16929b` were independently fetched here.
The previous task's 649 local tests/fmt/clippy at189f214 and unpushed `ce785c9`
are handoff reports; this checkout neither obtained nor regenerated `ce785c9`.


The next retained raw preLink unit represents newly added HeadChords in an actual
editing-owner registry with nullable geometry and lazy Measure placement. Eighteen
Do/Undo/Redo task boundaries and forty-two original listener records match the
explicit saved-owner control. First AdditionTask has no Part/Measure; first
Containment supplies the actual Head and placement; undo keeps the lazy reference
and redo appends to the Measure after vertex listeners. A late failure restores
Graph, real Chords, editing Chords, Measures, Voices and expectedDuration before
the same sequence can be retried. This is native atomic failure validation, not
an assertion that the original Java UI exception handler performs atomic rollback.

Callback Graph/getter state and typed owner state at every task boundary are
compared separately. Full owner membership inside every callback, integration
of editing Chords into ordinary Part/Measure/Voice reprocessing, canonical mirrors,
empty Voice/Part movement variants and controller conflicting-relation tasks
are pending. Original new-target setup warms actual Part/Staff before removing
only the target Chord; existing-target observations are unchanged. Public CLI
admission and global completion flags remain guarded/false. Automatic source
Stem extent still differs by five pixels; this is not full-image correction parity.
See `evidence/musicxml-headstem-prelink-new/TRY.md` and `NEXT-DEPENDENCY.md`.


A further small unit moves the actual editing HeadChord registry into the retained
Graph, so SIG callbacks own live member/stem views and lazy placement. New-Chord
typed members/stem/bounds, cached Measure reference and actual Measure membership
now agree inside all forty-two original listener records, including unplaced
initial addition, Containment entry/return and retained-reference redo registration.
Graph clone rollback now restores that same registry; no report rebuild or
callback-before clock reset is used. General normal Chord/Part/Voice callback
membership and ordinary reprocessing remain pending; CLI guards stay in place.
See `evidence/musicxml-headstem-prelink-callbacks/TRY.md` and `NEXT-DEPENDENCY.md`.


The next small unit preserves the actual empty Voice when the last Chord leaves
Measure.voices. Cached Chords retain a distinct native owner key to the moved
Voice and its old SlotVoice entries; undo does not restore Measure Voice-list
membership. A new independent original single-Chord Voice control compares twelve
raw task boundaries and thirty listener records. Native late failure restores
actual Voices/Chords/Graph and clocks, then retries/undoes/redoes the same sequence
without another retained Voice allocation. Earlier original cases are unchanged.
General reassignment/reused musical voice numbers, Part/Measure/Voice integration,
ordinary reprocessing/mirrors/controller/epilog/history/PAGE/persistence remain
pending. Public CLI and all completion flags stay guarded/false; the five-pixel
source Stem extent discrepancy is still separate from saved numeric controls.
See `evidence/musicxml-headstem-prelink-voice/TRY.md` and `NEXT-DEPENDENCY.md`.


Another small unit fixes explicit Voice reassignment after musical number reuse.
An independent original control creates new live Voice2 while detached old Voice2
still exists, and setVoice(new2) changes actual ownership despite equal numbers.
The previous native number-only shortcut reproduced a mismatch; detached owner
binding now prevents that shortcut and is replaced on explicit live assignment.
New2 gains the actual Chord list entry and keeps slot2; old2 retains slot1. Source
Slot/time/expected clocks are not reset or copied between those Voice owners.
Twelve task boundaries/thirty listeners plus explicit rebind states agree; late
failed redo restores actual current/retired Voice clocks and Chords before retry.
Other purge/rename/history identity paths, general Part/Measure/Voice callback and
reprocessing integration, mirrors/controller/epilog/PAGE/persistence and automatic
Stem extent parity remain pending. Public CLI/global flags stay guarded/false.
See `evidence/musicxml-headstem-prelink-voice-reuse/TRY.md` and `NEXT-DEPENDENCY.md`.


A further retained-owner unit fixes live Voice renaming with overlapping old Chord
lists. The original constructor/setVoice changes a reference without pruning its
old list; Measure.swapVoiceId renames Voice objects, so those old entries cannot
select Chord projections. The prior native list-based update reproduced chord0=2
against original1. Stable actual Voice identities now drive assignment, renaming
and retirement, while staged copies/rollback retain those identities. A fifth
independent original control compares twelve raw preLink task boundaries/thirty
listeners, original reference equality within callbacks, and explicit setup plus
post-retirement number reuse states. The preceding four cases are unchanged.
Native failure restores overlapping lists, actual references/Chords/Voices and
slot/termination/excess/expected clocks, then retries the same Do/Undo/Redo sequence.

This unit does not integrate all normal Chord/Part/Voice lists into callbacks or
complete purge/reset/history and ordinary reprocessing. CLI/global guards remain,
and automatic Stem extent still differs by five pixels. Existing refinement
controls keep their original expected observations; their setup now renames/binds
the actual native Voice instead of changing only numeric metadata. See
`evidence/musicxml-headstem-prelink-voice-swap/TRY.md` and `NEXT-DEPENDENCY.md`.


The next small lifecycle unit follows original Measure.removeVoiceChord's early
stop. Two actual Voice lists can retain the same Chord after setVoice. The first
empty Voice is removed and stops the loop; later lists remain through Do/Undo and
can be removed on Redo. Native reproduced deletion of both owners at once and now
uses successful membership removal/first-empty order. A sixth original control
adds twelve task boundaries/thirty listener records plus three setup states; old
five observations are unchanged. Cached old/new slots and original reference
equality are compared in callbacks. Late native failure restores both actual
Voices/lists/identities, Chords and clocks before the same sequence retries with
retained counts1/1/2. Remaining normal container/purge/reset/history owner paths,
Part/Measure/Voice callback/reprocessing integration, mirrors/controller/PAGE/
persistence and source Stem extent parity are pending; global/public CLI guards
stay false/guarded. See `evidence/musicxml-headstem-prelink-voice-stop/TRY.md` and
`NEXT-DEPENDENCY.md`.


A common actual Voice owner now retains the object dropped by refineSystem's
purge and by empty-Chord removal. Two independent original controls compare
sixteen states: real checkDuration yields-5/8 or+1/2/excess1/2, resetChords leaves
old slot/Chord references intact, purge removes only live-list membership,
re-add/re-purge reuses the same owner, and new Voice1 has distinct empty slots.
Native old-object loss was reproduced before the fix. Later-Measure refinement
failure keeps actual Graph/Voices/Chords/expected/slot/manual state and identities;
repair/retry succeeds. Source-derived genuine Measure/stack reset evidence is
already saved but native reset/history, normal container retirement and cached
beam/tie/NextInVoice Voice getter integration remain next units. No full original
SIG equality or automatic Stem extent parity is claimed; public/global guards
stay false/guarded. See `evidence/musicxml-voice-refine-retirement/TRY.md` and
`NEXT-DEPENDENCY.md`.


Actual Voice retention now covers genuine Measure.resetRhythm/Chord.resetTiming
and separate stack reset. Both original controls compare24 boundary states,
including unchanged16 prior purge/re-add/number-reuse observations. Measure
reset preserves old actual Voice bodies/slots/termination/excess, Chord duration,
manual state and stack timing; only legitimate Chord Voice/Slot/time references
are cleared. Stack reset separately clears actual/excess/slots while expected1
survives. Rebuilt Voice1 has a third identity. Native loss of the reused owner was
reproduced before the fix. A later invalid Chord fails after staged retirement
and earlier resets without changing actual Graph/Chords/Voice identities or
clocks; repair/retry matches the original boundary. General rhythm/history,
normal container retirement, cached beam/tie/NextInVoice getters and broader
Part/controller/PAGE/persistence integration remain pending. Public/global guards
stay false/guarded; no full SIG or automatic Stem extent parity is claimed. See
`evidence/musicxml-voice-rhythm-reset/TRY.md` and `NEXT-DEPENDENCY.md`.


Normal music_containers/Measure.removeVoiceChord now uses the same common
actual retained Voice owner as task removal, purge and Measure reset. Two new
independent original timed Measure callbacks compare12 method boundaries:
consecutive removals finally drop registration while old four slots, signed
termination-5/8 or+1/2/excess1/2, cached Chord Voice references and manual seed
survive. Original SIG removal is separately compared in existing exclusive/
shared controls; this does not claim full SIG or inside-listener equality.
Native old-object loss was reproduced before the fix. A later invalid Chord
fails refresh after deletion detection without changing actual Chords/Graph/
Voice identities, manual or expected/slot/termination/excess fields; repair/retry
retains one actual owner. General Part/Measure callbacks/history, retired Voice
getters through beam/tie/NextInVoice and broader controller/PAGE/persistence stay
pending. Public/global guards remain false/guarded. See
`evidence/musicxml-voice-container-retirement/TRY.md` and `NEXT-DEPENDENCY.md`.


Cached retired Voice lookup now follows actual identity through live and
retained owners. Two new original NextInVoice controls compare8 boundaries:
purge leaves old empty Voice7/slots/termination/excess referenced by previous
Measure Chords, NextInVoice renames target1 to7, repeat keeps7, then preferred2
wins. Native live-list-only refusal was reproduced before the fix. A malformed
later preferred ID fails after staged purge and actual reference-based swap,
without changing actual Graph/Voices/identities/Chords/manual relations or
expected/slot/termination/excess; repair/retry agrees. Common tie/BeamGroup
lookup uses this getter and existing live/null original controls remain in
regression. Dedicated retired beam/tie comparisons are pending, not a current
parity claim. General Part/callback/history/controller/PAGE/persistence and
five-pixel Stem extent remain pending; public/global guards stay false/guarded.
See `evidence/musicxml-voice-retired-next/TRY.md` and `NEXT-DEPENDENCY.md`.


Dedicated retired incoming tie comparison now covers two original controls20
boundaries. Production8744085 already agrees: cached old Voice7 is selected even
with distinct fresh7 in the live list; null first-source follows the next tie,
fresh7 becomes1 in refinement, restoring the old cached reference selects7.
NextInVoice then imposes1 and preferred2 wins. Actual Voice objects, source Head/
Chord and target reference identity, old four slots/termination/excess, manual
relations, clocks and legitimate Measure reset are compared. A late preferred
failure after staged incoming tie swap preserves actual owners/Chords/clocks;
repair/retry matches original. This checkpoint adds original proof and targeted
regressions without inventing a production fix. Dedicated retired BeamGroup,
automatic curve recognition and broader Part/controller/history/PAGE/persistence
remain pending; public/global guards stay false/guarded. See
`evidence/musicxml-voice-retired-tie/TRY.md` and `NEXT-DEPENDENCY.md`.


Actual retired BeamGroup comparison adds two independent original controls26
boundaries. Empty old7 retains six slots and signed termination/excess. The
actual group first-Chord getter and predecessor chord5 keep this cached owner,
rather than substituting last old-Voice Chord9. Fresh Voice construction
propagates ownership over four previous-Measure group Chords; fresh7 becomes1,
beam1 overrides tie2, Next2 then preferred1 win. Restored old live3 with an
overlapping member list is valid original HIGH sorting geometry, and re-purge
updates the same retained body. Null/restored cached references and genuine
Measure reset retain actual objects, expected/stack clocks and manual relations.

Two native discrepancies were reproduced and fixed: group getter relied on
live member lists instead of actual cached identity, and HIGH rejected a
reassigned Chord remaining in an old list based on number inequality. Existing
System/Part/Measure/staff and actual getter metadata guards remain. Late
preferred-ID/null/reset failures preserve actual owners, typed Chords and
clocks. Original refinement after reset throws NullPointerException; native
atomic guard behavior is not Java exception rollback parity. General selected
rhythm/history/Part/controller/PAGE/persistence and five-pixel Stem extent
remain pending, with public/global completion flags false/guarded. See
`evidence/musicxml-voice-retired-beam/TRY.md` and `NEXT-DEPENDENCY.md`.


Genuine Measure reset followed by ordinary selected stack rhythm now compares
four independently loaded original cases20 method boundaries. Actual
MeasureRhythm.process constructs two new Voice bodies and seven CompoundSlots
before stack slot generation; actual StackRhythm.process agrees with the
separate six-slot/actual-duration sequence before system refinement. Empty old7,
support2 and distinct fresh1 retain original six slots, termination/excess,
manual relations and source/target Chord identity. Cached expectedDuration is
preserved. The initial native null CompoundSlot references were reproduced;
common mapper constructor binding and new real Slot identity retain object
semantics when stack slot numbers are reused. Genuine reset is the only point
that clears cached Chord Voice/Slot/time references.

OwnedMusic.reprocess_stack_rhythm stages actual Graph/Voice/Measure state while
retaining actual Chord storage and untouched Measure Slot storage. Actual member
failure after staged reset preserves all owner/clock/manual/identity state and
retries against the original full-stack state. The same mapper is connected to
the existing image/selected export path; six manual HeadStem Link/Unlink actions
also exercise the new stack backend with original exact7/64, underfilled Voice
termination/divisions and native XML. Full MeasureRhythm implicit tuplet/rest
passes, multipart stacks and Page/controller/history integration remain pending;
public/global flags stay false/guarded. See
`evidence/musicxml-voice-reset-rhythm/TRY.md` and `NEXT-DEPENDENCY.md`.


Original PageRhythm.reprocessStack after genuine Measure reset adds two
independent controls six method boundaries. Its actual stack rebuild then
system refinement agrees with the earlier decomposed result; old Voice/Slot
bodies, signed termination/excess, cached expectedDuration and actual BeamGroup/
tie/Next identities survive. OwnedMusic.reprocess_page_stack_rhythm runs that
ordinary ordering as one native transaction. Late malformed preferred failure
preserves every actual Chord/Slot/Voice identity, old clocks/manual relations,
layout/report and untouched Slot storage; repair/retry agrees. This guard's
atomic rollback is not Java malformed-type/exception rollback parity.

The existing selected editing/export impact uses the common post-stack system
refinement routine; each selected stack still refines immediately, including
repeated systems. Six HeadStem actions exercise the Page backend with original
exact7/64, shortage/divisions and native XML. Original Reset executed-source
hashes remain pinned to8ae5b66. General MeasureRhythm passes, multipart stacks,
controller/history/PAGE/project/persistence remain pending; public/global
flags stay false/guarded. See `evidence/musicxml-voice-page-rhythm/TRY.md` and
`NEXT-DEPENDENCY.md`.


One independent null-unselected-source original Page control adds three method
boundaries. Actual next-stack rhythm constructs four fresh Voice bodies and
four Slots, actual duration5/8 and shortages-3/8/-1/2; real Chord time/Voice/Slot
references and old target bodies are observed before the original system
refinement throws NullPointerException. Production2a3bed1 already agrees with
that separate rebuild and refuses the null predecessor. Its public Page
transaction instead preserves all pre-call actual owner/clock/manual/Slot state
on failure; this is native atomic guard policy, not Java exception rollback
parity. No production fix or source invention is required. Seven relevant
native targets and fmt/clippy pass; previous XML/XSD is explicitly at2a3bed1,
whose production code is unchanged, without duplicate execution. Full selected
impact ordering and other nonUI dependencies remain pending and global flags
false/guarded. See `evidence/musicxml-voice-page-null-rhythm/TRY.md` and
`NEXT-DEPENDENCY.md`.


Actual source-first selected rhythm impact adds four independent original cases
14 boundaries: separate Page stack calls and actual RhythmsStep.impact(Collection)
with source,target,source agree. Original LinkedHashSet first-encounter order
suppresses the duplicate actual stack; debug calls confirm two selected Page
calls per case, with system refinement after each. Real old/new source/target
Voice bodies, cached expected/old clocks, manual relations and actual Chord/Slot
identity remain compared. Target has four new Voices/four Slots and actual5/8.

OwnedMusic.reprocess_selected_stack_rhythms stages the whole first-encounter
selection, then commits Graph/Voices and only selected timing. Later actual
member failure preserves real Chords/Slot storage/Voice identities and all old
clock/manual/layout/report state; repair/retry agrees with original impact.
Target-first [2,1] retains the null guard without order repair or source invention.
Single-stack APIs share this transaction; normal selected editing/export shares
its actual mapper and post-stack refinement. Public selected CLI render bridge,
full controller/task impacts and other nonUI dependencies remain pending;
global flags stay false/guarded. See
`evidence/musicxml-voice-selected-rhythm/TRY.md` and `NEXT-DEPENDENCY.md`.

### Retained selected stack CLI bridge

`audiveris-musicxml --reprocess-stacks MEASURE,...` now runs the existing admitted
Image selected rhythm path in first encounter order with duplicate suppression,
per-stack actual system refinement and real Voice/Slot XML binding. The original
order proof is retained at a4060de. New transactional coverage keeps original
actual Chords/Measures and untouched Slot storage, and restores actual owners,
old identities, expected clocks and XML/report on a later member or final XML
validation failure. Prior admitted dot correction evidence survives an explicit
selected pass. Existing Image/correction/fine-grid guards and all global/full
nonUI flags remain unchanged. See evidence/musicxml-selected-cli-rhythm; next is
original pre-reset BeamRest clear/detect preparation and implicit tuplet passes.

### Pre-reset BeamRest clear and actual RestChord cache lifetime

An explicit actual-measure clear backend now removes automatic BeamRest edges
and retains manual registration identity without resetting rhythm clocks or
creating Voices. RestChord beam-list cache objects survive clear and legitimate
Measure reset, and change only on actual cache invalidation, matching six
original controls/30 boundaries. A later typed endpoint failure rolls back staged
removal with actual Chords/Slots/Voice clocks and cache identities intact. This
backend stays separate from normal rhythm until BeamGroup interleaved-rest
detection is compared; implicit tuplets, generalized controller/tasks and the
remaining nonUI ledger are pending. See evidence/musicxml-beam-rest-clear.

### Actual linked rest candidates before detection

The actual-owner candidate query matches pinned getLinkedRests in two original
cases/eight boundaries. Incoming relations precede outgoing; layered Rest-only
closure keeps insertion order, cycles and foreign-Measure/null-Voice references.
The detector's actual Next-only whitelist excludes Same-only links and removes
the direct Separate blacklist after closure. Manual BeamRest changes actual
group seeds while preserving the old empty Rest beam-cache object. Genuine
Measure reset keeps old Voice/cache objects and clears only legitimate Chord
clock/Slot ownership. Missing actual Rest owners refuse transactionally without
placeholder Voice IDs or report reconstruction. Nearest-beam geometry, addition
callbacks and plain lookup remain next before full clear/detect can join normal
rhythm; all full/public completion flags stay false. See
evidence/musicxml-beam-rest-linked.

### Retained measured Skew and nearest interleaved-rest beam

Native Page now carries measured SheetSkew to its actual item owner instead of
recovering it from a mutable report or defaulting a missing measurement to zero.
The existing zero-skew admission remains guarded. Actual-measure nearest queries
match pinned full-only/skewed-intersection/inclusive-endpoint/strict-member-tie
semantics across two original controls/432 queries, retaining all old Chord/
Slot/Voice/cache clocks and identities. Later invalid member geometry refuses
transactionally. Original fresh BEAMS medians match native live precision;
the saved book's JAXB one-decimal boundary is explicitly separate, without
rounding the actual native owner. BeamRest addition callbacks and plain lookup
remain next before full clear/detect/implicit-tuplet preparation can join normal
rhythm. All full/public flags remain false. See evidence/musicxml-beam-rest-nearest.

### Actual BeamRest addition before rhythm reset

The explicit actual-measure backend now registers fresh full-beam/rest relations
and performs original addition callbacks without changing Chord/Voice/Slot clocks
or the retained Rest beam-list object. Parallel additions preserve the old manual
relation identity. Two original short/excess controls compare 24 states and 28
callback entry/exit observations, including ties-even dy, strict Scale pixel gap
boundaries, Skew and live side-support abnormal transitions. A later missing
actual member rolls back the entire staged batch, retaining physical owners,
manual flags, expected/end/excess and unused registration identity. Clear/reset/
invalidation boundaries remain distinct. Plain lookup, complete clear/detect,
implicit tuplets and the full nonUI ledger remain pending, with admission and
completion flags false. See evidence/musicxml-beam-rest-added.

### Plain rest candidates from actual Measure membership

Actual group measures are collected from beam/member chord encounter order,
then stably ordered by actual measure left in the admitted single-staff part.
Real Measure rest insertion order supplies plain candidates after original
black/white subtraction. Two original controls/ten states show a manual foreign
BeamRest expanding the group to the foreign actual Measure, without replacing
its null Voice or the held empty Rest cache. Collection preserves actual owners,
clocks and manual relation identities across clear and legitimate source reset.
A missing later foreign plain owner refuses and rolls back instead of silently
omitting that rest. Plain lookup polygons, competing removals/additions, full
clear/detect and implicit tuplets remain pending before normal rhythm; general
multipart/nullable owners and the full nonUI ledger stay unfinished, with all
full/public flags false. See evidence/musicxml-beam-rest-plain.

### Actual persistent Staff middle-line owners

GRID now retains the actual simplified StaffLine Curves in Staff geometry owners
before report publication, moving the same Staff/Curve references through Page,
items and Music. Actual middle-line reads ignore altered report staff/line fields
and refuse missing real Staff ownership without clock/Slot/Voice changes.
Thirteen original source samples and forty-five declared real Staff samples
compare 1/2/4/5/6-line lower-middle selection, interpolation/extrapolation and
ties-even integer yAt. Original StaffLine's +0.5px construction boundary and
explicit yAt(int) reflection are recorded separately from unchanged native
source precision. Selected success/later-failure/retry retains Staff/Curve refs.
Actual plain lookup polygons/competing callbacks, mutable Staff/controller
lifecycle, full clear/detect and implicit tuples remain pending; full/public
flags stay false. See evidence/musicxml-staff-midline.

### Actual cached-Chord polygon lookup geometry

Explicit actual-group lookup geometry now reads held Chord head/tail/bounds and
actual persistent Staff/Curve identity. Original middle-line extension, opposite
direction rounded intersections, strict x overlap and even-odd Polygon rectangle
contact match560 original private lookup decisions plus500 boundary controls.
Main median includes Hook/Small with first equal-width ownership; nearest remains
full Beam only. Cached Rest/Voice/Slot refs and clocks persist; a missing late
actual member refuses transactionally and retries on the same owners. Automatic
Stem5px extent discrepancy stays recorded separately without coercion. Competing
manual removal callbacks/fresh addition, full clear/detect/implicit tuples, mutable
Staff/Rest and full nonUI remain pending, with all full/public flags false. See
evidence/musicxml-beam-rest-polygon.

### Actual competing BeamRest removal and fresh addition

Explicit plain lookup now removes losing manual/automatic relations in original
live encounter order before fresh automatic registration. Original36 scenarios/
80 observations compare white priority, nearest-double versus rounded-dy refusal,
skewed sibling contains/contact, reinserted old relation identity/order, side
callback abnormal transitions and earlier-removal/later-rejection retention.
Callback clocks, physical Chord/Staff/Curve/Voice/Slot and Rest-cache refs persist.
Late actual-member failure after earlier staged removal/addition rolls back with
manual bodies and unused relation IDs held for same-owner repair/retry. Fresh
original observations are after callbacks; native entry retains original clocks
and independent added-callback proof. Full white-before-plain detect and original
implicit-tuplet removal-before-clear/detect/reset preparation remain next before
ordinary selected integration. Re-add/controller/history and general nullable/
mutable/multipart/full nonUI remain pending; all full/public flags stay false.
See evidence/musicxml-beam-rest-competition.

Captured actual BeamGroup Chords now drive complete explicit white-before-plain
detection on retained owners. Ten pinned original short/excess controls and20
post-callback observations compare strict endpoints/null, Next/Separate priority,
existing RestChord capture, held HeadChord bounds sorting and original NaN->int0
intersection. Late plain-member failure after white addition restores relations,
clocks and actual physical owners for same-owner retry. Sixteen unique targets,
fmt/clippy/build, existing63+correction8 and exact7/64 XML bytes/XSD pass.
Ordinary selected complete prep still requires original implicit Tuplet removal
BEFORE clear/detect/reset, then later implicit generation/merge ownership.
This is an intermediate checkpoint; authorized next work continues. Full/nonUI
and public flags remain false, original task/work/unobtainedce785c9 held.
See evidence/musicxml-beam-rest-detect.

Original implicit Tuplet preparation now retains real Measure insertion order and
admitted Stack unassigned references. Two short/excess controls compare ten
remove/clear/detect/reset boundaries and six post-removal callbacks with removed
SIG vertex still registered, containers already detached and old clocks held.
Manual implicit signs are removed; explicit/manual and Stack-only null-source
implicit signs survive. Live Chord duration/end follows first actual2/3 or4/6
factor; ordinary XML tuplet admission stays guarded. Combined late captured-Rest
member failure rolls back tuplets container storage, actual Chords and clocks.
Seven unique targets, fmt/clippy/build, default63+correction8 and exact7/64 XML
bytes/XSD pass. Next borrowed real-owner complete prep connection must preserve
existing selected admission/rollback; implicit generation/merge and full ledger
remain pending. Full nonUI/public flags false; same task/work/unobtainedce held.
See evidence/musicxml-tuplet-preparation.

Complete compared preparation now borrows actual Chord/Measure/Staff/Curve
owners in ordinary image and selected Stack processing after Stack reset before
Measure reset. Actual Book/Chord caches bind before initial rhythm, required by
real group getters. Tuplet containers commit/rollback with selected Graph/clocks.
Two original no-prior-reset Stack controls compare six boundaries and six
post-removal callbacks, fresh Voice/Slot construction before system refinement,
then refinement. Late target failure restores original Tuplet collection storage,
manual objects and physical Chord/old Slot/Voice clocks for same-owner retry.
Seventeen unique targets, fmt/clippy/build and existing63+correction8/exact7/64
XML bytes/XSD pass. Original Java expectedDuration object identity is observed;
native keeps its exact Fraction value. Actual TupletGenerator object reuse/group
extension/split, fresh generation, second pass/merge and full ledger remain next.
Full/public completion false; work189/original task/unobtainedce retained.
See evidence/musicxml-selected-preparation.

Actual Tuplet reuse phase and borrowed group extension/first-pivot split now
compare22 pinned short/excess scenarios. Supplied/incident live encounter returns
actual explicit/manual/Stack-only Tuplet immediately, including before a later
null. Original GraphListener records zero graph changes; old Voice/Slot and ending/
excess clocks remain. Beam members alone are deduplicated, initial/lone duplicates
remain, stable integer centers and one split preserve actual group refs. Late
actual physical Chord mismatch rolls back staged getters for same-owner retry.
Nine unique targets, fmt/clippy/build, existing63+correction8 and exact7/64 XML
bytes/XSD pass. None is the existing-object selector phase only; font dimensions/
bounds, fresh implicit callbacks/containers/relations, generation inspection/force/
merge/second pass and full nonUI ledger remain next. Full/public flags false;
work189/original task/unobtainedce unchanged. See evidence/musicxml-tuplet-reuse.

Tuplet existing/new pre-add placement now compares120 pinned short/excess
controls over6 explicit control-local family values. Source-measured immutable
FontSymbol dimensions use base4*interline, independent of calibrated HEADS:1536
rows/all6 families/two shapes/scales1..128,11,663 bytes hash pinned. Native plans
actual supplied Chord cache direction votes/negative tie/default, duplicate size
votes and integer rectangle union/center/height margin. Geometry is declared on
real retained Chords to isolate placement; automatic Stem5px remains separate.
Late actual getter failure restores staged caches/Graph and same physical owners.
Early existing/manual object precedes mismatched font/later null. Seven unique
targets, fmt/clippy/build and existing63+correction8/exact7/64 XML bytes/XSD pass.
Original360 allocation callbacks/generated/removed states are held losslessly for
next dependency, not claimed as native allocation parity. Fresh added/container/
link lifecycle, lazy getters/inspect/force/merge/second pass and full ledger remain.
Explicit font resources do not infer actual SheetStub family ownership or implement
general/unbounded font layout. Full/public flags false/work189/original/unobtainedce
held. See evidence/musicxml-tuplet-placement and assets/tuplet-font-dimensions.

### Retained Tuplet lazy getters (2026-10-08, continuation of96859d8)

Actual cached unreduced factor4/6, base/Staff retention, cached Part across null
Staff, first incoming relation Voice/null and musical-number reuse are compared
at20 pinned original boundaries. Real Chord ownership and native Staff/Curve/
Voice/Slot references survive staged getter rollback. Shape variation is private
field control only: no original setShape workflow. Ordinary null-Staff/fresh
Tuplet/XML admission remains guarded. Evidence: musicxml-tuplet-getters. Raw
sans-dot separation, fresh allocation callbacks, force/inspect/merge/second pass
remain next; full nonUI completion remains false.

### Actual raw duration and partial Tuplet base/Part (continuation of667b399)

All34 original shape-duration aliases plus non-note null, ignored dots0/1/2/9,
actual Beam/Flag sum and exact mirrored VOID promotion are compared on retained
physical Chord owners. Raw primitive is independent of dot/Tuplet/measure-rest
wrapper; ordinary special shape image/XML admission remains guarded. Source
partial base caching differs from outer transaction rollback. Inherited Part
reads existing Staff without triggering lazy Staff; critical fresh Stack boundary
is retained. Evidence musicxml-tuplet-raw-base has560+10+6 original boundaries
and4 fresh base references. Fresh Tuplet allocation callbacks and force/inspect/
merge/second pass remain next; full nonUI completion remains false.

### Fresh implicit Tuplet callback lifecycle (continuation of4eebedd)

The preserved120 original generation cases/360 callbacks now compare native fresh
vertex-before-SIG, added unassigned Stack/Measure order, supplied-order fresh
parallel relations, getter cache order and removal while old real Chord/Voice/
Slot clocks stay intact. Failure restores nonempty/manual actual containers and
backing storage. Actual source Stack boundaries/null/other-Stack ownership follow
before force/inspect/merge/second pass and ordinary font/rhythm admission. Evidence
musicxml-tuplet-generation; full nonUI completion remains false.

### Actual Stack boundary/null/other-Stack Tuplet ownership (continuation of75c6f26)

Source22 controls/44 callbacks compare left-excluded/right-included actual
Measure ranges, null outside, and generator Measure differing from added's
unassigned Stack. The other Stack keeps a retired Tuplet reference after own
MeasureRhythm removal; no global cleanup normalization. Outer failure restores
all actual container storage including a prior retired other-Stack object.
Evidence musicxml-tuplet-stack. Force eligibility/sans-Tuplet/grouping, inspect/
merge/second pass and ordinary font binding remain; full nonUI false.

### Actual dotted sans-Tuplet Chord/Voice sum (continuation of36aa619)

Original144 Chord+16 Voice reads compare raw inputs, dots1/2/ignored9 and default
measure-rest-shape bypass while actual manual SIX relation/old clocks remain.
Six original reset/reprocess boundaries retain old Voice sums/physical Chords
and distinguish new objects reusing numbers1/2. Source unknown labels and Slot
identity are observed without ID-based ownership substitution. Evidence
musicxml-tuplet-sans. Beat grouping/force eligibility, rest-Voice/switch ownership,
inspect/merge/second pass/font binding remain; full nonUI false.

### Actual ordered Voice beat grouping (continuation of3d61c65)

Original96 private buildGroups/checkGroups controls compare raw beat3/2, one
nextTime increment per Chord, large gaps/backwards offsets,32 null failures and
4 HALF-to-QUARTER fallbacks across actual BeamGroups. Physical owner/Slot storage,
manual/relation identity and old clocks remain intact. Explicit beat planning is
not automatic signature/force admission. Evidence musicxml-tuplet-beat; force
generation/eligibility, inspect/merge/second pass/real font binding remain, full
nonUI completion false.

### Actual Voice force generation/reuse (continuation of539497a)

Original4 cases/8 private force calls/26 fresh callbacks compare composed actual
beat grouping/BeamGroup extension/split/generation and repeated same-object result
refs. Second force reuses objects with no callbacks. Outer later allocation failure
restores earlier completed Tuplet/real container storage/Chord/Slot clocks; all-
existing calls retain unchanged storage. Explicit source beat/font controls precede
ordinary binding. Evidence musicxml-tuplet-force. Eligibility/inspect/merge/second
pass and real signature/font/switch ownership remain; full nonUI completion false.

### Actual ordinary Voice implicit Tuplet find/force (continuation of5ed8453)

Original32 find controls compare ZERO/null/nonzero first start, exact dotted sans-
Tuplet ratio, actual Voice selection,18 fresh objects and8 null/zero expected
failures. Null/nonZERO starts bypass later sum/expected/beat/font without inventing
Voice IDs. Old physical Chord/Slot clocks/manual objects remain. Evidence
musicxml-tuplet-find. Rest-Voice constructor/empty live Voice boundary follows
before inspect/merge/second rhythm pass and real font/signature/switch integration;
full nonUI completion remains false.

### Actual Voice null-list reset and real measure-rest constructor (continuation of6cc3d2b)

Source two short/excess controls distinguish allocated-empty ZERO sum from
resetChords null-list NPE and registered-empty find index failure. Old Chord/Slot/
Voice clocks/manual ownership remain. Real WHOLE_REST/RestChord factory produces
actual ID3 and separate rest reference; before/after reset its null normal list
returns null duration and bypasses find before expected access. Native uses actual
nullable Voice state/current-retained identity and direct list lifecycle, without
Measure purge or number synthesis. Evidence musicxml-tuplet-find-inputs. Actual
Voice Tuplet getters/inspect-shrink/merge/second pass/signature/font/switch remain;
full nonUI complete false.

### Actual Voice Tuplet readers and pure Measure purge (continuation of3515653)

Original5 cases/25 states/50 Voice rows compare first relation encounter vs any-
relation last Chord, absent/null successor, allocated-empty/null getter distinction,
retired old object and restored current encounter order. Pure purge shares existing
refinement removal without triggering sort/rename early. Physical Chord/Slot clocks,
manual objects/expected/report remain. Evidence musicxml-voice-tuplet-sequence.
InspectVoicesEnd/shrink real candidate owners/merge/second pass and actual resource
binding remain; full nonUI complete false.

### Actual narrow Slot owner clocks (continuation of8d172a6)

Original4 constructor controls/20 states distinguish shared member objects from
distinct objects with reused ID7 and compounds with reused ID3. Native member clones
keep Arc identity and clock reads/writes follow actual owners, preserving duplicate
Chord concatenation/first-containing lookup/backrefs/old Voice and Stack Slot clocks.
Ambiguous numeric validation cannot merge distinct objects. Evidence
musicxml-narrow-slot-owners. Actual in-pass shrink/inspect/merge/second rhythm pass,
fully typed validation resource binding and general ledger remain; full nonUI false.

### Existing-Tuplet shrink on actual candidates (continuation ofb8318fa)

Original7 direct private-method controls use actual generator and actual retrieved
narrow/compound owners. Native retains candidate Arcs/clocks, matches exact7/64,
stop/default-first/BeamGroup extension/reuse, Chord push before narrow update, no
fresh callbacks and held Voice/Stack Slot/end/excess/manual/expected. Baseline Slot
observation now compares held Arcs across same-number old/new objects. Original
late cast leaves earlier1/12 and1/6 narrow clocks; native outer rollback restores
actual Graph/Chord/candidate/container state and retries. Evidence
musicxml-tuplet-shrink-existing. Fresh callback and nullable fallback/inspect/merge/
second pass/normal resource binding remain; full nonUI false.

Fresh actual-candidate shrink now captures original registration/ChordTuplet callback
clocks before later exact push/narrow assignments: five source controls/four fresh
Tuplets/26 callbacks, genuine old Voice/StackSlots/end/excess/expected/manual state.
A second-link failure restores outer native actual containers/cache/Chord/candidate
clocks and permits retry. Explicit resource/cached geometry controls follow actual
candidate retrieval. Evidence musicxml-tuplet-shrink-fresh; nullable fallback and
inspect/merge/second pass/ordinary resource binding remain; full/public false.

Nullable shrink now follows actual stopChord Voice across live/retained owners,
returns on legitimate NULL/empty lists, preserves source absent/empty/reversed
range semantics without inventing Voice IDs. Nine original controls retain actual
Chord/StackSlot/Voice/expected/end/excess/manual identity and candidate clocks with
zero new Graph callbacks. Evidence musicxml-tuplet-shrink-nullable. Signed inspect,
merge/second pass and ordinary resource binding remain; full/public false.

Actual Voice tail inspection now selects first implicit/last ANY-related/successor
owners and expected-null/zero-normal/signed exact3/2 eligibility. Ten source controls
include both differences negative and a later Voice null-time failure after first
Voice1/12,1/6,7/24,5/12,2/3 narrow changes; native outer rollback restores actual
Graph/Chord/candidate/container/manual state. Evidence musicxml-tuplet-inspect-end.
Collection removal/stale-predecessor merge, second pass and real ordinary resources
remain; full/public false.

Ordered Tuplet collection removal now preserves explicit/implicit/manual actual
objects, duplicate retired references, caller Measure->Stack Part/unassigned detach,
explicit actual center lookup (NULL is valid), implicit other-Stack retired ownership
and18 old-clock callbacks across8 source controls. A later NULL member preserves
source partial removal separately from native actual Graph/container/cache/manual
rollback and prefix retry. Raw unassigned collections and actual owner copies are
observed rather than inferred from unions/report. Evidence musicxml-tuplet-remove-
collection. Stale-predecessor merge, second pass and ordinary resources remain;
shared multipart/extensive relations/full nonUI/public false.

Actual stale-predecessor Tuplet merge now preserves captured current Voice order,
same/different/NULL real BeamGroups and removed previous object/group after fresh
or reused generation. Eight original controls/41 ordered callbacks compare actual
old and fresh signs/shape/bounds/manual relations, raw containers and old Chord/
Voice/StackSlot clocks. Source late registration partial state is distinguished
from native outer graph/cache/container/physical-owner rollback and retry. Evidence
musicxml-tuplet-merge. Capture-old/find-new/remove-old second pass and ordinary
resource binding remain; full/public nonUI false. No new authentication or environment.

First-pass implicit recheck captures old actual Measure signs before find creates
or reuses signs, removes exactly that capture only for nonempty returns, preserves
returned duplicates/retired objects, and reaches the next genuine reset without
clearing callback clocks. Ten original controls/20 callbacks cover old/new coexist,
first/pass2/switch/NULL versus zero expected and source partial callback failures
versus native actual owner/cache/storage rollback/retry. Evidence musicxml-tuplet-
second-pass. Retaining map candidates and real ordinary resources remain; full/
public nonUI false. Checkpoints are interim and work continues within authorization.

Real image quality goal: actual GRID Part/Staff/Curve and area ownership now reaches
beam builders, with source containing-area/integer-distance/core-curve neighborhood
selection. Two real systems/60 points/18 beam centers match actual original Staff
and Part identities, including duplicated same-Part references. Unmodified An Laura
now passes the multi-Part beam guard and stops at foundation overlap class admission.
Evidence real-beam-neighborhood records this interim connection and targeted XML/XSD
regression. Independent source-score comparison finds 73/101 exact events in the
92-event original export; voice/chord/notation remain unscored. Real-image quality is
not accepted, and recognition-required overlap/graph/rhythm/output work continues.

Retained real-image GRID BarConnector area and Brace glyph overlap now matches
11 original actual objects and11 directional/support controls. NULL connector glyphs
are legitimate; NoExclusion/BarConnection Support suppress exclusions, BarGroup does
not. Existing manual relation bodies and repeat identity remain. Evidence real-grid-
overlap. Full An Laura now reaches the subsequent foundation kind admission guard;
that retained GRID reduction and later recognition/rhythm/output remain. No real
image MusicXML or quality acceptance is claimed.

Actual GRID Brace/BarConnector bodies now reach foundation checks with original
frozen/contextual grades. Eight source automatic weak controls and actual full-image
Brace retirement preserve Part/Staff/Curve Arc/storage and physical registry; manual
structural removal and late bad endpoint roll back prefix/relations. Evidence real-
grid-foundation. Full An Laura progresses to the multi-Staff ledger reduction guard;
real image MusicXML/quality acceptance remains false and work continues.

Constructor-produced Ledger memberships are now held through PAGE/STEMS with
actual Staff/Curve Arc and canonical Graph body references, not report reconstruction.
Fresh real An Laura original LEDGERS/HEADS frames match two systems/six Staffs/26
bodies and maps. Three declared shared/duplicate/retired-reference frames preserve
first body index, differing map indices, last Staff-only removal and body identity.
Evidence real-ledger-memberships. The ordinary multi-Staff ledger guard is still
pending, alongside later real image recognition/rhythm/output/quality acceptance.

Ordinary multi-Staff Ledger pruning now uses retained actual Staff maps in original
Staff/index/member order, including removed next-line bodies and duplicate aliases.
Four original declared controls and fresh unchanged image HEADS comparison match26
bodies/six Staffs/10 removals with no shared case. Late malformed Head failure keeps
actual maps/body identities intact; selected rollback and XML/XSD regressions pass.
Evidence real-ledger-reduction. Full An Laura passes existing STEMS foundations and
now stops at MEASURES music_single_barlines_head_only_scope_required. Grouped GRID
barlines/retained multi-Staff piano Part and later recognition/rhythm/output remain;
real image quality is not accepted. Continue the revised image-quality goal.

Actual grouped physical barlines now keep the same Staff/Curve abscissae, directed
BarGroup member keys and manual relation identities through MEASURES admission.
Eight original constructor cases and live same-image original TEXTS-to-MEASURES
observations match18 groups/21 members across six Staffs. Frozen aggregation,
automatic ensemble status, intrinsic grade and null four-member style match.
The actual full-image library path still retains four Part objects with Staff
counts1/2/1/2 when upstream report GRID ownership is blank; it now reaches
music_multistaff_parts_pending. This is not a full-image CLI/XML success.
Evidence real-barline-groups records necessary focused and XML/XSD regressions.
Next connect actual piano PartBarline/Measure and common system Stack ownership.
Real-image quality remains unaccepted; unrelated API/editor work stays excluded.

Real PartBarline and Measure layout now holds the same actual GRID Part/Staff/Curve
objects, ordered StaffBarline member objects and common system Stack references.
Original same-image live normal-dispatch MEASURES comparison matches four Parts,
eight Measures and four Stacks, including the piano's per-Staff abscissae and the
previous right bar's identity as the next effective left bar. Late noncanonical
Staff failure stages graph/container allocation without touching clocks. Existing
single-Staff continuation consumes this layout. The full-image library reaches
music_multistaff_chord_dispatch_pending; real native CLI XML/quality is still
unaccepted. Shared Stack geometry does not claim shared rhythm/Slot completion.
Evidence real-part-measures. Continue actual piano Chord dispatch and downstream
recognition/rhythm/output, excluding unrelated APIs under the revised image goal.

Actual Part-wide Chord construction/dispatch now keeps the piano's same multi-Staff
Part and retained Measure geometry instead of separate Staff measures. Original
normal same-image CHORDS comparison matches67 Chords/94 Heads, all member pitch/
Staff/Part/Measure relationships and Chord/Stem/head/tail geometry. Original default
ReductionStep.refineHeadEnd is connected with the same physical glyph/cache Arc,
held geometry and unchanged constructor-time relation consistency. All67 Stem
glyph bounds/weight/width match; median residual is below4e-12 pixels. A late bad
Head anchor preserves prefix/cache/manual relation bodies, and later cache sync
retains internally certified refined geometry. Evidence real-head-chords. This is
not accepted real-image MusicXML quality; SYMBOLS multi-Staff membership, common
Stack rhythm/voices/output and independent quality acceptance remain.
Full unchanged image CLI now reaches SYMBOLS music_symbols_curved_or_skew_staff_pending
in171.66 seconds with no XML; the retained actual Staff/Curve ordinate and skew
lookup is the next recognition dependency. All image-quality flags remain false.

SYMBOLS header erasure now evaluates the retained actual Staff curves at header
stop and fills the original inclusive rectangle. Original PageCleaner invocation
on the unchanged image records six rectangles and a full-page pixel mask; native
compares those boundaries and actual Part/Staff refs without rebuilding geometry
from report GRID ownership.
Evidence real-symbol-headers. This removes the header's flat-line assumption only;
later cleaner raster/weak optionals and actual symbol Staff/pitch selection remain,
along with full real-image XML and independent musical quality acceptance.
Full unchanged image CLI v9 passes the curved header and reaches
music_cleaner_beam_ledger_other_shapes_pending in172.82 seconds, with no XML.
The next unit is actual physical GRID Inter area/glyph erasure; quality stays false.

Actual SYMBOLS structural Inter erasure now uses retained vertical median/ribbon
width instead of an inflated Barline bounding box. Original actual visitor
comparison matches all21 Barline and9 null-glyph BarConnector rasters, reading
the same canonical graph bodies and preserving graph/relations/clocks. A preliminary Brace
branch was closed after actual CLI exposed missing canonical GRID glyph ownership.
Retaining that constructor-produced physical registry is next. Evidence real-symbol-areas. Only the existing
normalized thin rectangle contour branch is admitted; general slopes and real
SYMBOLS ensemble/weak-optional order remain subsequent recognition work.
Real image MusicXML and musical quality acceptance remain false.

GRID physical glyphs now begin as typed objects at registration and move through
Brace integration/PAGE/STEMS on the same Arcs. Recognition passes the original
registry, while the report-only facade remains separate. Actual Brace erasure
resolves the canonical GRID node's glyph ID in that retained physical registry;
it no longer expects an absent Beam-format glyph projection. The real-image
target blanks report glyphs, matches two original Brace run tables, and preserves
all PAGE/STEMS physical Arcs, graph bodies and registry storage. Evidence
real-grid-glyphs. Recognition/rhythm/output and independent quality remain pending.
Full unchanged image CLI v11 passes physical Brace erasure and reaches
beam PageCleaner sloping/wider stroke raster pending in170.56 seconds, no XML.
The next image dependency is14 sloping contours among18 original Beam/Hook bodies,
then real Chord-based cleaner visitor order. Real-image quality remains false.

Original sloping Beam/Hook cleaner raster is connected through normalized convex
miter contours and raw area fill, without a bounding-box replacement. Eighteen
actual native graph Beam/Hook bodies match original median/height and every pixel
row; fourteen cross normalized rows. Eight independent declared boundary/slope
controls and old rectangle/Hook contracts pass. Canonical graph and constructor
physical registry stay held. Evidence real-beam-raster. Wider/collapsed/bevel
contours retain explicit dependencies if required; musical quality remains false.
Full unchanged image CLI v12 passes Beam contours and reaches
music_filter_weak_head_optionals_pending in173.7 seconds, no XML.
Continue actual strong Chord visitor order; quality remains false.
