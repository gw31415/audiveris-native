# 実譜面画像 → MusicXML 品質受入

更新: 2026-10-08。ユーザーの変更した終了条件を適用する。

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
実ライブラリ経路は `music_multistaff_chord_dispatch_pending` に到達した。
この新切断点は全画像のライブラリ対象検証であり、native全画像CLIの成功を示さない。
XMLを出力できない。staff消去済み同名開発fixtureでの `no_regular_lines` は画素が異なり、
実譜面評価から除外する。元の画像と歴史的rendererのexit qualificationは保持する。
今回Linuxで原版を実行しており、新たなMac処理・再render・環境移動はしていない。

## 改訂した残工程

1. 複数part/staffでのbeam周辺の実所有選択は接続済み。次の実入力切断点である
   MEASURESの保持済みGRID/grouped barlineは接続済み。実piano PartのMeasure/Stack幾何所有も接続済み。次に複段Chord dispatchと共有rhythm時計を接続し、
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
