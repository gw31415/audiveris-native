# BEAMS: native BeamStructure geometry

Audiveris 5.11.0 / upstream `9e1e55cd2746037d059345881c53e6a6754bffbd` の
`BeamStructure` 幾何処理を Rust へ移植した段階です。実行時に Java、外部プロセス、
fixture の期待値、ネットワークを使いません。

## 今回の範囲

- 垂直 run glyph → `SectionFactory` / `JunctionRatioPolicy.DEFAULT` の section 分解
- section 上下境界の `BasicLine` fitting、長さ順の global slope、傾斜差の除外
- ordinate offset と重み付き重心による border grouping、hook 幅の除外
- border の対応検索・端点補完、global mean distance、median / height / item span
- `compareSlopes`、`adjustSides`、`extendMiddleLines`、`splitLines`、`computeJitter`
- `BeamsBuilder.ItemParameters` の default scale 換算と明示的な入力・計算量制限

`BeamInter` / `BeamHookInter` の採点と初期生成は別モジュールの
[BEAM-IMPACTS](BEAM-IMPACTS.md) と [BEAM-CANDIDATES](BEAM-CANDIDATES.md)
で検証します。この幾何単体の比較から候補採否を推定していません。
beam の結合・延長、後段 hook、グループ、MultipleRests、音符・拍・MusicXML は未完了です。

## 保持した境界条件

- border の上端は run start、下端は stop + 1。jitter の下端は stop
- section slope は長さ降順、同長は元の順序。global slope の平均との差が 0.3 を超えると
  累積を止め、その global slope から 0.3 を超える section を除く
- border grouping / lookup の margin は typical height × 0.8。lookup は近い順ではなく
  ordinate 昇順で最初の許容 entry を選ぶ
- `allowBorderCreation=false` の pinned default のみを実装。見つからない境界は生成しない
- `LineUtil` は slope 補間に置き換えず、1000-unit の垂直線との determinant intersection
  とその浮動小数点演算順を維持
- `compareSlopes` は幅が maxHookWidth より大きい line だけを使い、該当する隣同士を比較
- side adjustment は距離が minBeamWidthLow 未満のときだけ実施
- `extendMiddleLines` は 3 本以上なら名前にかかわらず全 line を左右端まで延長
- stuck-beam split は推定本数が 3 以上でも 2 本を生成。新しい item は生成しない
- side / middle adjustment は line median のみを変え、既存 item median は保持
- jitter の corner margin と probe ordinate は Java `Math.rint` と同じ ties-to-even
- section polygon containment は垂直 run の half-open pixel cell と同じ境界で判定

## 独立比較の内容

`tests/beam_structure.rs` は 2 種類の比較を別々に行います。

1. `fixtures/official-beam-prolog` の 13 画像内で original `BeamStructure` が実際に
   呼ばれた 144 glyph。section、distance、各中間 line/item、幅、傾斜差、jitter を比較
   - float 3,412 値、fdlibm 修正後の最大絶対誤差 `0`
   - observer が bytecode から再現した precheck のみの 339 glyph はこの比較から除外
   - original geometry 呼出しは post-prolog の別診断。production dispatch decision の
     独立観測と同一視しない
2. original JAR のメソッドを直接呼ぶ合成 glyph 279 ケース
   - float 11,512 値、fdlibm 修正後の最大絶対誤差 `0`
   - border 不成立 111、stuck split 35、複数 line 43、複数 item 13 ケースを含む

加えて module unit tests で、明示した初期 line を original に与える 27 ケースの
strict-edge 比較を行います。ここでは `computeLines` を呼んでおらず、その旨を fixture
に記録しています。hook 幅の等号、side margin の前後、2 本 / 3 本の延長、split の高さと
重量、jitter の空サンプリングを含みます。malformed input、予算超過、再実行も検査します。

比較許容差は float の**絶対値 1e-8**。整数・個数・分岐結果は一致を要求します。
bitwise 同一とは主張しません。

### 数値差の原因と独立 review での修正

当初の platform `hypot` は Java `Math.hypot` と 1 ULP 異なり、Laura glyph index 51
で global distance の差 `7.888584785398e-10` が観測されました。原実装の raw-moment
二乗距離計算が小さな正規化差を増幅するためです。当時の診断は
`evidence/beam-structure/numerical/` に履歴として保持しています。

独立 review では seed 固定の 1,000 glyph を追加し、1e-8 を超える数値差 31 値と、
line height が `16` の前後にずれることで split 本数が変わる 1 ケースを確認しました。
さらに 18 run の 2 ケースへ縮小し、誤 split と split 見逃しの両方向を保存しました。

修正は許容差の拡大や値の丸めではなく、別 license の `audiveris-area` に保存した
OpenJDK `FdLibm.Hypot` の演算順・high-word splitting を native 化したものです。
元 source は取得済み local JDK archive だけを使用し、GPL-2.0-only と Classpath
exception を保持しています。runtime で Java を呼びません。

修正後は従来の 144 + 279 glyph、追加 1,002 glyph がすべて最大絶対差 0 で一致しました。
追加 corpus は section / median / height / item / jitter と各 branch の本数を比較し、
`tests/beam_structure_review.rs` で回帰検査します。5,321 個の独立 Java hypot vector
でも finite/infinite の bit 一致と NaN 分類を確認しています。これは検査した corpus の
結果であり、任意入力での bitwise 同一性を包括的に証明したとは主張しません。
provenance と修正前後の記録は `evidence/beam-structure-review/` を参照してください。

## API と native 制限

`BeamStructure::new(&GlyphPixels, ItemParameters)` で構築し、`compute_lines()` を 1 回呼びます。
結果は `Result<Option<f64>, String>` で、境界不成立は `Ok(None)`、入力や予算の問題は
`Err`。成功後は `lines()`、`sections()`、`width()` と各幾何メソッドを使用できます。
`width()` は空なら `None`。`compute_jitter` は元実装の undefined line 例外を明示的な
`Err("undefined_beam_structure_line")` として返します。

原実装が同じ instance への `computeLines` 再呼出しで line を重複追加しうる点は、native
phase IR では one-shot contract として明示的に拒否します。計算途中の予算エラーで line
の部分結果は公開しません。制限は upstream 自体の制限ではありません。

- 垂直 run のみ。canonical sorted / non-overlapping / non-adjacent run を要求
- glyph 面積 4,000,000 以下、各寸法と絶対座標範囲 `[0, 16384]`
- run 250,000 以下、default section 4,096、border line 1,024、item 100,000 以下
- SectionFactory の隣接 run 比較、border 対応・item 探索に明示的 work budget
- default work budget 16,000,000。`with_limits` は bounded な変更を許可
- finite な nonnegative parameter、positive typical height、core width ≥ 2 と順序関係を検証

## 再現

既存のローカル公式配布物と JDK だけを使います。ダウンロードや Mac 利用は行いません。
出力先は新規 directory が必要です。元 fixture は上書きしません。

```sh
python3 scripts/run_beam_structure_controls.py \
  --oracle-home ../linux-oracle \
  --output evidence/generated/beam-structure-new
python3 scripts/run_beam_structure_review.py \
  --oracle-home ../linux-oracle \
  --output evidence/generated/beam-structure-review-new
cargo test --locked --test beam_structure --test beam_structure_review -- --nocapture
cargo test --locked -p audiveris-area matches_original_java_hypot_vectors
cargo test --locked --lib beam_structure::tests -- --nocapture
cargo clippy --workspace --locked --all-targets -- -D warnings
```

`BeamStructureControlProbe.java` は reflection を使う開発専用 observer です。geometry は
hash 固定の未改変 official JAR を呼びます。compile-only の代替実装を runtime classpath
へ入れる工程はありません。generator、observer、JAR、入力・出力の SHA-256 と実行記録は
`evidence/beam-structure/original-controls/provenance.json` を参照してください。

## 出典・帰属

Audiveris の元実装は AGPL-3.0-or-later。`src/beam_structure.rs` 冒頭に pin と帰属を明記。
今回の翻訳は手元にある original JAR の bytecode を使用しました。追加のネットワーク source retrieval は不要です。`fixtures/beam-structure-source` に元の BeamStructure /
ItemParameters と補助 geometry class のローカル bytecode dump を保持します。
