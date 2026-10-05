# Template font boundary audit — Audiveris 5.11.0

**推奨は、当面「公式生成のversion付きtemplate asset + Rust数値core」です。**
任意サイズのnative描画は別の互換性課題として残します。今回のpure Rust描画実験は
15ケース中6ケースだけが画素一致でした。assetの読込・照合はJava不要で、公式Javaと
45件のscoreが完全一致しました。これは検証した15 assetの一致であり、全サイズ・全形状・
他OSのAWT出力・符頭認識全体との互換性を意味しません。

## 実物で確認した境界

Pinは `9e1e55cd2746037d059345881c53e6a6754bffbd`、公式5.11.0配布JARを変更せず使用。
JAR/font hash、生成環境、定数、asset hashは [manifest.json](manifest.json) に固定しました。
本番Rust coreへのJava依存は追加していません。Javaは参照assetを作る開発時oracleだけです。

1. **Fontとサイズ** — `Staff.getHeadPointSize` → `MusicFont.getHeadPointSize`。
   `MusicFontScale`があれば `rint(staffInterline / mainInterline * storedPointSize)`、
   なければ `4 * rint(staffInterline * headRatio)`、headRatio既定1.0。
   さらに`BlackHeadSizer`は単独黒符頭候補の幅順中央50%から平均幅を求め、
   `buildMusicFontScale`へ渡す。ここでは2つのAWT `TextLayout`の黒符頭幅を計測し
   point sizeを補間する。つまり4の倍数だけのassetでは不足する。
   公式測定で幅6/8/10/13/16/20px → size20/27/34/44/54/68。
   Java `rint`はties-to-even。測定幅からpoint sizeまで忠実にする場合、AWTの幅計測も
   version付きmetric表として保存するか、native計測との一致を別途検証する必要がある。
2. **文字・fallback** — Bravuraにはbackupなし。Leland/Primus/FinaleJazz等は別familyへ
   fallbackし得る。shape→Unicodeのmappingもfamily別。実験はBravuraのみ：
   black E0A4、void E0A3、whole E0A2、cross E0A9、filled diamond E0DB。
   small headsは元shapeの0.67倍変換で、専用文字を読むだけでは再現できない。
3. **Canvasと描画** — `TemplateSymbol`は`TextLayout.getBounds`からtight boundsを取得。
   周囲のroomは `2*ceil(8*(pointSize/80)/3)`、canvasはceil、glyphは中央配置。
   small headでは一部で整数fat boundsを使う。`OmrFont.frc`はAA/fractional metrics有効、
   `ShapeSymbol.buildImage`のGraphics2Dでは一般AAをOFFに設定し`TextLayout.draw`する。
   native側でAAのboolを合わせるだけではこの一連の挙動を保証できない。
4. **Slim boundsとanchor** — alpha>=140かつred<140を前景に二値化。
   2画素以上の前景がある最外行・列をslim boundsとする。
   CENTER/MIDDLE_LEFT/MIDDLE_RIGHTは全shape、stemlessのwhole/breve系はこの3点だけ。
   他shapeは左右stem計6点を追加（横位置は幅の10%内側）。通常のblack/voidは左右で
   上下位置が非対称、cross・diamond・triangleには専用分岐。整数丸めの前に浮動小数点
   offsetsを保持する必要がある。1pxの描画差でslim boundsとanchorが変わる。
5. **Keypoints** — 原実装の3/4 chamfer距離→shape別hole seedの近傍3×3探索・flood fill
   →孤立背景調整→hole距離を負にする。左上がholeになると2倍解像度で再試行。
   reduceはratio×ratio距離平均をさらにratioで割り、rintし、±1を0にする。
   `dist <= rint(8*pointSize/80)`を行順に収録。任意の輪郭bitmapから単純な中心holeを
   仮定しても、circle-Xやbreve-crossを含む公式shape集合には対応できない。

**重要な実測:** WHOLE_NOTEは24ptで48pt、44ptで88ptへ公式が再描画・縮小した。
そのため最初の24/44pt bitmapに対する公式template自身のscoreも0にならない
（約0.2075/0.2712）。assetには最初のbitmapだけでなく、最終keypointsを保存すべき。

## 小規模native実験

同じ未変更Bravura **1.392**（512,924 bytes、SIL OFL 1.1）のCFF輪郭を
`ttf-parser 0.25.1`で取り出し、`tiny-skia 0.11.4`で描画した。native normal依存は9 crate、
すべて既存cacheからofflineビルド。実験で使うbitmapは5形状×24/44/64ptだけ。
canvasのroom式は公式に合わせ、輪郭tight boundsで中央配置。位置を後からfitしたり
画像を拡縮して差を消したりしていない。この実装は**unhinted**であり、AWTのhinting・
サブピクセル位相・metric量子化を完全に模倣した実装ではない。

|比較|AA OFF|AA ON、alpha>=140|
|---|---:|---:|
|画素完全一致|4/15|6/15|
|全ケースXOR画素合計|133|119|
|最低foreground IoU|0.4681|0.5000|
|slim bounds一致|10/15|10/15|
|最大score差の絶対値|0.25926|0.23333|

scoreは**同一の公式template**を公式bitmap／native bitmapへ照合した差。native生成の
keypointsを公式と比較した結果ではない。44pt crossで差が大きく、AA ONでも23画素が
相違し、公式score0に対してnative bitmapのscore0.23333。44pt blackも15画素相違、
slim boundsは公式[2,3,13,10]に対してnative[2,2,13,11]。64ptでは0〜5画素差。
この1組のrendererの結果から、すべてのnative font engineが不可能とは結論しない。
原因をhintingだけと断定していない。エンジンと配置phaseの切り分けは未実施。

[全数値](comparison.csv)・[機械可読結果](comparison.json)・[Rust実験](native/src/main.rs)。
公式assetの45 score（15形状サイズ×公式/AA OFF/AA ON入力）は既存Rust template coreで
**f64完全一致**。debug/releaseテスト済み。clippy/fmtも通過。

資源の参考値：30 bitmapを生成するwarm-cacheの単発process約6.9ms、maxRSS約2.55MiB、
release executable 694,496 bytes、最大RGBA canvas 2,992 bytes。この測定は負荷試験ではなく、
全font・大サイズ・並行入力の資源上限を保証しない。詳細は [resources.json](resources.json)。
`time -l`の一部OS統計はsandboxで取得できず、Python child rusageでRSSを採取した。

## 選択肢と最小案

|観点|native font生成|公式生成asset|
|---|---|---|
|忠実性|今回の描画は不一致。metric、位相、hinting、hole/reduce、small/fallbackが追加課題|指定した生成環境・サイズ・shapeの最終keypoints/anchorsを保持できる|
|サイズ対応|設計上は任意サイズへ展開可能だが、今回確認は3サイズだけ|収録した整数sizeのみ。未収録sizeを自動補間・丸めない|
|資源|font parser/rasterizerとfontデータ、canvas作業領域が必要|runtimeにfont parserもJavaも不要。15件JSONは39,251 bytes、gzip 5,855 bytes|
|更新|font engine/font/定数の更新で出力が動く可能性|再生成時に差分とhashを確認し新versionへ切替|
|権利表示|Audiveris由来ロジックAGPL、font OFL、crateの各license|AGPL由来コード表示とfont由来assetのOFL/copyrightを分けて保持|

最小構成は **exact-key asset lookup → 既存Rust `Template`**。keyはAudiveris commit、
font hash/family、point size、shape、生成定数・renderer環境のversion。assetにwidth/height、
fat/slim bounds、浮動小数点anchor、最終keypointsを格納。manifest hash・件数・座標・有限値・
点数budgetを検証してloadする。今回testはheaderと45 scoreを検証したPoCであり、
耐改ざんを含む完成したproduction loaderではない。

収録外は明示的に`UnsupportedTemplate`として返す。nearest sizeやbitmap拡縮を互換経路へ
混ぜない。まず実入力で現れる**正確な整数point size**と必要shapeだけをoracleで追加し、
CI等の開発時に検証して配布assetを増やす。Javaをユーザー実行環境へ持ち込む必要はない。
完全native生成の開発は別featureに分け、同じhash付きgoldenへ比較する。

不足は、幅→point sizeのmetric表／native互換、全head形状・small変換・backup font、
残る整数サイズ、hole/reduce生成のRust化、他OS/JDK差の調査、production loaderのbudgetと
破損asset検査。今回15assetは**任意サイズ対応ではない**。

## 権利・再現手順

Audiveris由来のコード／保存したJava sourceはAGPL-3.0-or-later、著作権表示は保持。
未変更Bravura fontを同梱しており、正確なembedded copyrightとOFLを
[fixtures/BRAVURA-OFL.txt](fixtures/BRAVURA-OFL.txt)へ保存。font単体販売禁止、再配布時の
copyright/license同梱、改変fontのReserved Font Name制限を保持する。今回font自体は未改変。
生成keypoint集を「一般文書だから無条件に別license」と決めず、font由来assetとして元の
OFL表示を併記する保守的な配布案。fontのlicenseとAudiveris由来ソフトのAGPLを混同しない。
ttf-parserはMIT OR Apache-2.0、tiny-skiaはBSD-3-Clause。実際のlock内crateのlicense表と
原文（dev/test依存を含む）は [dependency-licenses.json](dependency-licenses.json) と
`third-party-licenses/`。native font案は対応するlicenseも配布物へ残す。

```sh
cd native
cargo run --offline --release -- ../fixtures/Bravura.otf ../fixtures/native
# Java oracle再生成はoracle/FontOracle.javaをJDK21でcompileし、
# pinされた公式5.11.0のJava25とapp/* classpathで実行する。
# args: fixtures/official fixtures/native
cargo test --offline
cargo test --offline --release
cargo clippy --offline --all-targets -- -D warnings
cd ..
python3 compare.py
```

test依存`audiveris-template-match`は前回deliverableと同じ親フォルダに置く。
通常のnative描画binaryはこれにもJavaにも依存しない。AWT oracleの全head catalog生成を
避けるため、今回はprivate Builderをreflectionで指定shapeだけ呼んだ。大規模全生成なし。

一次資料:

- [MusicFont.java](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/ui/symbol/MusicFont.java)
- [BlackHeadSizer.java](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/sheet/beam/BlackHeadSizer.java)
- [TemplateFactory.java](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/image/TemplateFactory.java)
- [TemplateSymbol.java](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/ui/symbol/TemplateSymbol.java)
- [ttf-parser 0.25.1 CFF outline API](https://docs.rs/ttf-parser/0.25.1/ttf_parser/struct.Face.html#method.outline_glyph)
- [tiny-skia 0.11.4](https://docs.rs/tiny-skia/0.11.4/tiny_skia/)

公式sourceの完全なpin付きコピーは`oracle/upstream/`、そのhashはmanifestに保存。
