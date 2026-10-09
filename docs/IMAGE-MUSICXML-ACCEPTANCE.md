# 実譜面画像 → MusicXML 品質受入

更新: 2026-10-09。ユーザーの変更した終了条件を適用する。

## ゴールと対象

少なくともPNG画像を入力し、native Rust CLIのみで譜面を認識してMusicXML4を
出力する。認識結果・原版XML・元のMuseScore音符をnativeへの入力にしない。
認識に必要な所有・関係・時計・callback処理は維持する。PDF/多形式変換、一般
編集CLI/UI/undo、project保存互換、認識に不要な原版全API再現は完了条件から外す。
過去の機能移植証拠は保持するが、その全API台帳を残作業として実行しない。

## 受入の方法

- 原版5.11.0/source9e1e55cdとnativeに**同じ未改変実譜面ページの画素**を与え、
  両方を画像から最後まで実行する。狭い合成譜面・staff消去済み中間画像・XML-loaded
  原版工程を実譜面認識の成功へ読み替えない。
- 既存CC0 OpenScore/LiederのAn Laura、Der Winter、Der Brautschmuckを基準候補とし、
  まず保存済みAn Laura全ページを通す。他2曲のLinuxでの画像化・原版exportを用意し、
  休符や独立譜面の確認を加える。曲・ページ・画素hash・原版設定は比較前に記録し、
  成功する音符だけの切出しやfixture別の正解埋込みをしない。
- part/staff/system/小節の欠落がなく、音高（臨時記号/調号含む）、正確な有理数の
  onsetと長さ、休符、和音の同時音、声部の連続性、拍子と小節境界を全イベントで比較する。
  beam/旗/付点/連符/tieなど音楽解釈に必要な関係は、その譜面にあるものを比較する。
  表示上の座標・XML element並び・ローカルvoice番号の違いは正規化し、意味を比較する。
- 同じ曲の独立CC0 .mscxを正解参照として使用し、原版とnativeそれぞれの欠落・追加・
  音高/音価/声部/小節/記号関係誤りを記録する。原版との一致だけで原版の誤りを正解に
  しない。nativeの各音楽品質指標が原版以上であることと、残差の実譜面上の確認が受入条件。
  現在はeventの正解比較のみ着手しており、声部連続性・和音群・記号関係の比較は未完。品質同等性を主張しない。
- native出力がMusicXML4 XSDに通り、独立readerで全part/小節/声部/イベントを読めること。
  原版より良い相違は独立正解と実譜面で説明する。未比較カテゴリを成功に含めない。
- 既存合成63イベント＋明示補正8とexact7/64 XML bytes/XSDは回帰として保持する。
  全649件の反復や全API達成は受入条件ではない。

## 最初の実証と切断点

[同一画像の基準](../evidence/real-image-quality-baseline/BASELINE.json)：An Laura全ページ
1489×2104、180dpi、元のRGBAページと同一のgray画素、2systems/voice+piano3staffの実曲。
現在の原版画像exportは成功（2parts×4小節、92note要素、和音継続27、3/4、beam45）。
これらは原版の認識件数であり、独立正解92件一致や品質受入を示さない。

同じ画像のnativeは複数Partのbeam周辺Staff選択を原版60地点と比較して接続した。
実BarConnector/Braceの重なり判定も接続し、現在はその先の
GRID整合と実Staff mapによる普通の複数Staff Ledger pruningも接続済み。
STEMSの既存経路を越え、MEASURESの保持済みgrouped barlineを原版18群/21memberと
比較して接続した。同じ実Part/StaffのPartBarline・8 Measure・4共有Stack幾何所有も接続し、
実ライブラリ経路はCHORDSの67 Chord/94 Head、全member/音高/Staff/Part/Measure所属、
Stem/Chord/head/tail幾何で原版に一致した。原版標準のREDUCTION Head-end refineも
元glyph/cache identityを保って接続した。実Staff曲線によるヘッダー消去も6 Staffの原版矩形/ピクセルと比較して接続済み。
実Barline/BarConnector30個の消去ラスタも原版と比較済み。
実画像CLI v10ではBraceへの未接続glyph参照が `invalid_beam_canonical_bounds` として
判明し、その試験分岐を閉じた。GRID constructorから実glyph registryを同じArcで保持してBrace消去へ接続済み。
実Beam/Hook18個の傾斜消去も原版全ピクセル行と比較して接続済み。
実Chord経由の弱Head消去も、67強Chord/94Headの強弱判定とbottom-up訪問順で原版一致し接続済み。
grade数値は最大0.000363878の差があり、数値同等性を主張しない。
実System領域・Staff/null/曲線音高も796地点と543 glyph中心で原版と比較して接続済み。
post-analysisの12仮想Ledger曲線をcallbackから保持し、物理Ledger削除後も維持する。
古い全マスクの全0バイト観測は無効と明示した。実Bravura51ptの整数pointSize/4を保持してHead消去余白を3pxから原版の2pxへ修正し、正しい原版全マスク3,132,856バイト・黒82,837画素が一致した。以前の109画素差は修正前証拠として保持する。
実System所属543 glyphから原版の176 cluster・最大23部品の7部品選択と788候補集合を比較し、2部品限定を解除した。原版自身のHashSet順は再観測で変わるため、候補集合と遭遇順固定の再帰契約を比較した。実譜面の記号生成・rhythm・XML品質受入はまだ未完。実画像CLI v16は166.87秒でpositive AlterHead linkガードへ進み、XMLは未出力。
実Alter factoryの17件（リンク13/未接続4）は原版の音高・中間状態・関係値と一致した。
これは実glyphに原版shape/gradeを与えた制御比較で、classifier全体の同等性ではない。
実画像CLI v17は170.86秒で複数StaffのDot所属ガードに到達した。保持済み実Part/Measure/Stack
によるDot登録を接続した。原版境界52地点と宣言Dot24操作は一致し、対象6件とXML/XSD回帰も成功。
実画像CLI v18は171.14秒でDotの曲線Staff判定に到達した。次は実曲線・実Ledger距離と所属選択、
実曲線・実Ledger距離は788実候補と45 Ledger制御、実Part Measure選択12地点で原版一致して接続済み。
対象6件とXML/XSD回帰成功。実画像CLI v19は177.52秒でinstant articulation判定に到達した。
実Staff近傍/Chordによるinstant staccato lookupと追加・リンク・除去callbackを接続した。
原版97候補lookupの正15、16実記号の制御状態が一致し、対象6件とXML/XSD回帰成功。
実画像CLI v20は172.0秒でrepeat判定ガードへ進んだ。次は原版instant repeatをstaccatoより
前で実Staff/Barlineから判定し、共有Stack Dot links／filter、後続記号reduction・共有rhythm・出力へ。
原版instant repeatをstaccatoより前へ接続し、97実dot候補の反復0と168宣言Barline
lookupを比較した。対象6件とXML/XSD回帰成功。実画像CLI v21は175.43秒で実AlterHead
supportのcontextualizationガードへ進み、次は支持付き臨時記号のweak purge、その後の
実container/共有rhythm/outputへ接続する。shared Stack全filterと正repeatの生成/pairは
後続に残す。実Alter支持のcontextualization/weak purgeは17実記号と204宣言条件で比較し、
実Head intrinsic gradeの残差最大2.22e-16、除去2個が一致して接続済み。対象6件とXML/XSD
回帰成功。実画像CLI v22は184.55秒でHead/Stem/Flagの外部relation projectionへ進み、
その原版比較が次の切断点。後続競合除去・共有container/rhythm・音高伝播と出力は未完。
原版最終XMLの8 staccatoはまだnative出力で比較できていない。
XMLはまだ出力できない。
この新切断点は全画像のライブラリ対象検証であり、native全画像CLIの成功を示さない。
XMLを出力できない。staff消去済み同名開発fixtureでの `no_regular_lines` は画素が異なり、
実譜面評価から除外する。元の画像と歴史的rendererのexit qualificationは保持する。
今回Linuxで原版を実行しており、新たなMac処理・再render・環境移動はしていない。

## 改訂した残工程

1. 複数part/staffでのbeam周辺の実所有選択は接続済み。次の実入力切断点である
   MEASURESの保持済みGRID/grouped barlineは接続済み。実piano PartのMeasure/Stack幾何所有も接続済み。複段Chord dispatchと原版標準Stem Head-end refineも接続済み。次にSYMBOLSの複段所属と共有rhythm時計を接続し、
   その後同じ実譜面のSYMBOLS/rhythm/output切断点を順に解消する。実画像のLedger
   共有補正は0件で一致し、真に共有する別入力では再所属補正を省略しない。
2. 調号/臨時記号、複段part、和音、実音価・小節・voice・必要な記号関係を画像から保つ。
   既に移植したrhythm内部処理は、実入力が要求する順に本経路へ接続する。reportから
   所有を再構築したり、callback前に旧時計を消したりしない。
3. 同じ所有をnative MusicXML出力までつなぎ、独立の実譜面・休符入力と原版/正解比較を
   完成させる。PNG以外の変換、編集・undo・project互換の工程は追加しない。
4. 意味のある小単位ごとに対象比較・必要なXML/XSD回帰・通常main commit/push/readbackを行う。
   最後に実譜面品質指標と残差を評価し、受入が成立した時点でゴール達成と報告する。

現状: **画像品質受入未達**。元task/環境/work189を保持し、未取得ce785c9を取得済み扱い・
置換・再生成しない。認証・通常Gitに実障害なし。小checkpointは途中報告として継続する。

初回の独立event比較: 原版92eventのうち73が元スコア101eventとstaff/小節/onset/
MIDI/音価/graceで厳密一致。precision0.79348、recall0.72277、F1=0.75648。
声部連続性・和音群・記号関係はこのevent multisetの指標では評価していない。
[全残差](../evidence/real-image-quality-baseline/ORIGINAL-EVENT-QUALITY.json)。
これは初期基準であり、native品質受入や最終品質判定ではない。
