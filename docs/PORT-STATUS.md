# 原Audiveris 5.11との機能同等性：移植台帳とCLI接続順

更新日: 2026-10-05（America/Los_Angeles）。対象原版は5.11.0、ソース `9e1e55cd2746037d059345881c53e6a6754bffbd`。

最終目標は、UIの表示・入力装置を除く、原版の譜面認識・解析・補正・保存・MusicXML出力機能の同等性です。原版機能を意図的に省略する予定はありません。通常五線・Bravura・ト音記号・単声などの現在の制限は、小チェックポイントで検証した範囲であり、最終スコープの縮小ではありません。GUI内にある処理も、譜面の意味・形状・関係・再計算に影響する部分は移植対象です。

台帳は現在の実コード・保存済み原版観測・原版クラスの対応に基づく機能群の棚卸しです。全原版メソッドの網羅監査が完了したという主張ではありません。未調査の細部は未対応に含め、工程ごとに細分化します。「部分」は条件指定kernel、保存XMLからの比較、画像CLI接続のいずれかが存在する状態で、原版工程全体の完成を意味しません。HEADS/STEMS/MusicXML/epilog/全自動処理の完了フラグはfalseです。

## 実装済みの具体的単位

- PNG 8-bit grayscale/PGM読取、原版の適応二値化・runs・classifier特徴量と149出力モデル推論。入出力形式・領域上限・全種類のページ処理は下表の部分対応です。
- 共有SIGの所有ID、glyph registry/cache、関係・削除履歴の保持、および保存された多数のGRID/HEADERS/BEAMS/HEADS/STEMSの比較単位。全SIG意味論の移植完了ではありません。
- 限定画像CLIで四分/二分音符と休符、第一下加線C4、複数小節・改行を認識し、nativeの結果からMusicXMLを生成。旧成功入力を保持しています。
- 今回の2部品四分休符の部品/union登録、RestInter・chord・LINKSへの接続。Rest/TimeWholeの非support・非frozen部分について原版の実画素overlapとexclusion選択を移植。

全回帰の件数・ソースSHAは段階ごとの検証要約で示します。通常workspace移行前のMANIFESTとLaura352heads/513exclusionsなどの報告は当時のソースSHAを保ち、新ソースで再実行した証拠に読み替えません。

直近の全回帰はsource `52943de`で583成功・0失敗・ignored 0。途中中断後は未完対象のみ再開し、176実行物を全照合。doc tests、対象fmt/clippy、別clone Linux build・20イベント・MusicXML4 XSDも通過。[検証記録](../evidence/musicxml-ordered-foundations/FULL-VALIDATION.json)。この後の説明だけのcommitは同じsrc/crates treeです。

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
| SYMBOLS / `SymbolsFilter`, `SymbolsBuilder`, `InterFactory`, `DotFactory` | 部分 | classifier・isolated/2部品rest/time・実加線へのdot距離検査と普通の1旗→stem接続あり。原版7部品上限のcluster、全形のfactory/shape checks、真正augmentation/staccato/repeat dot、flag/accidental/articulation/dynamics/ornament/数字等の生成とlateChecksが残る。149分類出力を全記号の意味接続済みと扱わない。 |
| RHYTHMS / `SlotsRetriever`, `ChordsMapper`, `StackRhythm`, `MeasureRhythm`, `PageRhythm`, `Voices` | 部分 | 認識済み拍子に応じた完全小節の単声の二段beam十六分/beam・旗付き八分/四分/二分イベントをnativeで計算（全画像比較は4/4と3/4）。一般slot/voice割当、他beam/flag音価、付点/連符、tie、pickup/不足超過小節、複声部/複段/途中拍子変更の原版処理は未接続。 |
| PAGE/score / `ScoreReduction`, logical parts | 部分 | 単ページ改行・小節番号あり。複数ページ継続、part同定/結合、楽器/移調、跨systemの声部/slur/tie、repeat構造等の一般score統合が残る。 |
| MusicXML / `PartwiseBuilder`, original proxymusic | 部分 | native MusicXML4 writer、限定イベント・system break・part名、schema/reimport比較あり。一般notation/direction/lyric/instrument/layout/score属性と.mxl圧縮、多ページexportの原版対応が残る。原版XMLの流用は認識完成に数えない。 |
| プロジェクト保存・再開 / `Book`, `Sheet`, stage invalidation | 部分 | stage reportsとGit完全復元あり。native認識projectの永続化・読込・編集後の必要工程再実行、原版.omrとの対応/互換性は未対応。Gitソース保存とは別機能。 |
| 補正 / `InterController`, tasks/history, editors | 未対応 | 下記CLI案の対象。GUIクラス内に存在することを理由に認識・補正ロジックを除外しない。 |
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
or removed chord members. The report at `limited_links.containers` also projects
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
