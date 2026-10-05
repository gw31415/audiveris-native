# 最新：C4加線付き1譜面のMusicXML接続

[CLI手順・限定範囲](checkpoints/sol-musicxml-ledger-cluster-20261005/CLI-GUIDE.md) / [Linux実行物](checkpoints/sol-musicxml-ledger-cluster-20261005/audiveris-musicxml-linux-x86_64.tar.gz) / [MANIFEST](checkpoints/sol-musicxml-ledger-cluster-20261005/MANIFEST.json)。対象回帰7件・fmt・clippy成功。新ソースの全回帰は未実行。旧544全回帰とreleaseは下記の当時の版として保持。新たな原版成功入力1件8イベントをnative画像処理で確認し、中央四分休符の拒否を維持。Library再探索・再upload無し。

# Audiveris native — 限定MusicXML CLI

Linuxで譜面画像から認識した音高・音価・休符・小節をMusicXML 4.0へ出力する限定版です。実行時にJavaや原版の認識XMLは使いません。

- [CLI手順・入力例・既知の受理制約・未対応一覧](checkpoints/sol-musicxml-half-rest-links-20261005/CLI-GUIDE.md)
- [Linux x86_64実行物・入力例・期待MusicXML](checkpoints/sol-musicxml-half-rest-links-20261005/audiveris-musicxml-linux-x86_64.tar.gz)
- [実装ソース・全544回帰・復元証跡のMANIFEST](checkpoints/sol-musicxml-half-rest-links-20261005/MANIFEST.json)

実装ソースは `a2bf13ca8da7afe0b9e41da491190558898f4b34`、tree `410d8bf30a826e17316cbaf4cc60a1cff656ea2e`、6,841ファイル。全544回帰・fmt・clippyをこのソースで確認済みです。文書更新ではソース・実行物を変更せず、再ビルド・全回帰は繰り返していません。

対応はBravura/線間11px/傾き0/通常五線/ト音記号/C major/明示4/4/単声/四分・二分音符と休符の限定配置。3独立合成譜面24イベントと、同じ二分休符譜面の2位置10イベントを区別して確認しています。中央四分休符の弱候補など受理できない配置も残ります。同じ中央配置の原版通常画像処理でも休符をG4音符へ誤認識することを確認し、nativeの安全な拒否を保持しています。実スキャン・別フォント・特殊staff・一般配置は未検証または未対応です。全Audiveris移植の完了とは扱わず、全体完了フラグはfalseです。

Gitで実装・Linux実行物を保全しています。今回のLibraryアップロードは結果不明で、同一原本の参照IDが未特定です。親の検索でも特定できなかったとのユーザー確認を受け、未確認のまま保持し、再探索・再アップロードは行いません。Git保全を基準に作業を継続します。テスト削減は次の指示を待ちます。

下記は過去のHEADS veto段階の記録です。現在のCLI到達点や未完一覧は上の手順書・MANIFESTを参照してください。

# Historical recovery checkpoint (2026-10-03)

This private repository preserves base archives, exact-source recovery bundles and validation receipts. Reconstruct the Rust source using the pinned manifest chain.

- Historical source: `df25de2ded65d3fc8a25898116ffaae03f56a0ba`; tree `12465ec2ea444299722b20a0bddedb88579799fe`; 4798 tracked files. Independent fresh restore verifies every tracked SHA256 and all eight changed files.
- Historical stage: HEADS pre-registration staff/ledger seed proposals now apply original frozen-bar and good-competitor veto before each template evaluation. Raw 418 proposals remain available; gated 358 proposals (328 staff + 30 ledger) match original Laura across 118 scanners and 1,137 seed/pitch pairs. These are proposals before range scans and registration.
- Independent original evaluation: 42,700 evals; 464 bar vetoes, 288 competitor vetoes, 680 central and 72 noncentral vetoes. Global obstacle selection, median/thickness/grade geometry, scanner slices, 279 bar polygons, mask/distance hashes and all proposal geometry/shapes/scores/grades match.
- Thirty unique focused tests passed; the three veto tests passed again after final additions. Formatting and all-target Clippy passed. Current full workspace suite was not rerun for this bounded stage.
- Preserved full baseline `b7c279eaa64b495fdf6aea4eaae1563925ba7bb7` / remote `910c7c8d4d763fadd279cf00c50dc912ee9559c2`: debug/release each passed 340 tests across 109 suites, failure/ignored zero, once each. Complete saved logs and receipts remain in the [baseline manifest](checkpoints/sol-head-ledger-search-20261003/MANIFEST.json).
- [Historical source/validation manifest](checkpoints/sol-head-veto-20261003/MANIFEST.json), including exact bundle, dependency order, original bytecode, logs and fresh recovery proof.

HEADS and full OMR remain unfinished: horizontal range search, glyph retrieval/HeadInter registration, duplicate/overlap purge, tally, stems/rhythm/MusicXML remain pending. Additional eligible area types without exact original area metadata fail explicitly; positive real-score rest competition is not demonstrated. All earlier archives and source checkpoints are preserved.
