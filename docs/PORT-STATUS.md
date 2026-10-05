# 原Audiveris 5.11との機能同等性：移植台帳とCLI接続順

更新日: 2026-10-05。対象原版は5.11.0、ソース `9e1e55cd2746037d059345881c53e6a6754bffbd`。

最終目標は、UIの表示・入力装置を除く、原版の譜面認識・解析・補正・保存・MusicXML出力機能の同等性です。原版機能を意図的に省略する予定はありません。通常五線・Bravura・ト音記号・C major・4/4・単声などの現在の制限は、小チェックポイントで検証した範囲であり、最終スコープの縮小ではありません。GUI内にある処理も、譜面の意味・形状・関係・再計算に影響する部分は移植対象です。

台帳は現在の実コード・保存済み原版観測・原版クラスの対応に基づく機能群の棚卸しです。全原版メソッドの網羅監査が完了したという主張ではありません。未調査の細部は未対応に含め、工程ごとに細分化します。「部分」は条件指定kernel、保存XMLからの比較、画像CLI接続のいずれかが存在する状態で、原版工程全体の完成を意味しません。HEADS/STEMS/MusicXML/epilog/全自動処理の完了フラグはfalseです。

## 実装済みの具体的単位

- PNG 8-bit grayscale/PGM読取、原版の適応二値化・runs・classifier特徴量と149出力モデル推論。入出力形式・領域上限・全種類のページ処理は下表の部分対応です。
- 共有SIGの所有ID、glyph registry/cache、関係・削除履歴の保持、および保存された多数のGRID/HEADERS/BEAMS/HEADS/STEMSの比較単位。全SIG意味論の移植完了ではありません。
- 限定画像CLIで四分/二分音符と休符、第一下加線C4、複数小節・改行を認識し、nativeの結果からMusicXMLを生成。旧成功入力を保持しています。
- 今回の2部品四分休符の部品/union登録、RestInter・chord・LINKSへの接続。Rest/TimeWholeの非support・非frozen部分について原版の実画素overlapとexclusion選択を移植。

全回帰の件数・ソースSHA・Linux実行物は各MANIFESTが正本です。過去のLaura352heads/513exclusionsなどの報告は当時のソースSHAを保ち、新ソースで再実行した証拠に読み替えません。

## 機能別の現在位置と欠落

| 原版機能・具体的依存 | 状態 | nativeの現在位置 / 残り |
|---|---|---|
| LOAD / `ImageLoading`, `Book`, `Sheet` | 部分 | grayscale PNG/PGM単画像・複数独立画像CLIあり。RGB/alpha/JPEG/TIFF/PDF・複数ページbook・原版画像変換/ページ選択は未接続。4M画素等は現ガードで、最終的な省略ではない。 |
| BINARY/SCALE / adaptive filter, `ScaleBuilder` | 部分 | 原版式・histogram/peak・多数kernelあり。一般画像の全設定/分解能/失敗状態と下流伝播を広げる必要がある。 |
| GRID / `GridBuilder`, `Staff`, `SystemInfo`, parts | 部分 | sections/filament/通常staff/system/parts・cue第二パスの比較あり。非zero skew、特殊staff、実スキャンの全工程一致は未検証。 |
| HEADERS / `HeadersBuilder`, clef/key/time関連クラス | 部分 | 音部記号・調号・拍子の探索/状態に既存部品と原版比較あり。画像→MusicXMLはG-clef/4/4のガードが残る。固定HEADER調号を実SIG/実glyphへ接続し、G major1画像のF♯を原版と一致確認済み。途中変更、他clef、一般調号と臨時記号、他拍子の意味伝播を接続する。 |
| STEM_SEEDS/BEAMS/LEDGERS / 各Builder | 部分 | 種/beam/ledgerの既存比較あり。positive beam-origin STEMS、cue beam、一般加線音高・共有staffの全工程は未接続。 |
| HEADS / `HeadsBuilder`, template catalog | 部分 | 24形catalog等のkernel、Laura/制御比較あり。24形全ページ伝播・別font/サイズ・特殊staff・実画像の全面比較は未完。 |
| STEMS / `StemBuilder`, `StemsStep`, head/beam linkers | 部分 | head由来画像stumps/chunks/items/glyph/cache/live SIGの接続あり。beamからの自動選択/接続、一般cleaner/epilog、32sources/items制約の除去・検証が残る。 |
| REDUCTION/LINKS / `SigReducer`, `SIGraph`, `SymbolsLinker`, `InterCleaner` | 部分 | 共有SIG、一部head/stem cleaner、弱休符/空chord削除、今回のRest/TimeWhole overlap/exclusionあり。全Interのsupport contextual grade、互換性、frozen、各関係のconsistency、反復reductionとcallbacks/epilogは未接続。原版に`reduceSymbols`というメソッドはなく、この経路は`reduceLinks`。 |
| TEXTS / `TextsStep`, Tesseract, words/sentences | 未対応 | 歌詞・題名/作曲者・tempo/メトロノーム・文字role認識、staff/partとの関連付け・OCR修正。原版のローカルOCR依存に沿う。外部AI設計に置換しない。 |
| MEASURES/CHORDS / `MeasuresBuilder`, `ChordsBuilder`, `MeasureFiller` | 部分 | 単staff/単stemの小節・chord形成あり。複段part、和音、stemless whole/breve、cross-staff、複数stem、repeat/double bar等の一般形成/補正が残る。 |
| CURVES / `CurvesStep`, slur/wedge/ending/rehearsal builders | 未対応 | 音楽上のslur/tie/wedge/ending/rehearsal認識・接続。nativeの`curves.rs`はGRID曲線補間であり、この工程の移植証拠ではない。 |
| SYMBOLS / `SymbolsFilter`, `SymbolsBuilder`, `InterFactory`, `DotFactory` | 部分 | classifier・isolated/2部品rest/time・実加線へのdot距離検査あり。原版7部品上限のcluster、全形のfactory/shape checks、真正augmentation/staccato/repeat dot、flag/accidental/articulation/dynamics/ornament/数字等の生成とlateChecksが残る。149分類出力を全記号の意味接続済みと扱わない。 |
| RHYTHMS / `SlotsRetriever`, `ChordsMapper`, `StackRhythm`, `MeasureRhythm`, `PageRhythm`, `Voices` | 部分 | 完全4/4小節の単声四分/二分イベントをnativeで計算。一般slot/voice割当、beam/flag音価、付点/連符、tie、pickup/不足超過小節、複声部/複段/途中拍子変更の原版処理は未接続。 |
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

実コード上の直近の切断箇所は、`key_state.rs`にKeyInter/fifths生成がある一方、`music_symbols.rs`のPageCleaner接続が非nullのheader key/timeを拒否し、`music_xml.rs`がG-clef/4/4を限定、note alterを0固定、writerをG-clef/4/4固定にしている点です。これらを単にガード解除するのではなく、原版の有効調号・臨時記号・拍子依存を繋いでから実画像で確認します。原版`HeadInter.getAccidental/getMeasureAccidental/getAlteration`の公開APIも実クラスで確認済みで、調号fifthsだけの出力ではなくこの音のalter決定依存を追います。

1. **今回のcompound rest/SIG checkpointを固定**。実装済み部分、画像全工程比較、保存XML段階比較、条件指定競合kernelを分けて保存する。
2. **直近の未接続はHEADERの意味伝播**。既存clef/key/time部品を実measure/音高・臨時記号/拍子状態とexportへ繋ぐ。まず既存資料で原版が成功する小さな一入力の他調号を選び、調号だけを出力するのではなく実音のalterまで比較する。入力は未選定・未検証であり、今回の成果には数えない。調号/accidentalの原版依存が必要なら最小部分を合わせて接続する。
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

次の具体的未接続は固定HEADERのTimePair/TimeWholeの所有SIGと小節拍数への伝播、他clefの音高/export接続です。SYMBOLSで初めて認識された調号、cancel/途中調号変更、一般臨時記号はその後に接続します。これらを最終機能から省略しません。HEADS/STEMS/MusicXML/epilog完了フラグはfalseです。
