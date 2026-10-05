# Audiveris 5.11.0 — 公式headless batch oracle

本家の未変更配布jarと同梱Javaを使い、元画像からLOAD → BINARY → SCALE → GRIDを実行できた。Native実装のscale/section/filament IRはJava入力に使っていない。

| 入力 | 各stage | GRIDの本家結果 |
|---|---|---|
| OpenScore Lieder「An Laura」原RGBA PNG、1489×2104、180 DPI | 全4stage成功 | **6 staff / 30 lines / 2 systems** |
| 同ページの既存比較用gray画素を無損失PNG化 | 全4stage成功 | **6 staff / 30 lines / 2 systems** |
| 自作640×420、2五線とバー線 | 全4stage成功 | **2 staff / 10 lines / 1 system** |
| 既存320×320、バー線なし2五線 | LOAD/BINARY/SCALE成功、GRID exit 1 | `StepException: No system found`。失敗参照として保存 |

`golden.json` に16回分のstage結果・入力SHA・実行成否・scale・staff/system所属・各StaffLine点列と太さ・画像画素SHAを保存した。成功はexit 0と `.omr/book.xml` の完了stepの両方で判定する。合成失敗を成功goldenに混ぜない。

## そのまま使う場所

- 実行・保存script: `run_oracle.py`（標準Python）。既存runは上書きせず要約を再読込。
- golden再生成: `build_reference.py`（Pillowは画像ファイルの画素decodeだけに使用）。
- 全実行コマンド、exit、時間: `runs/<input>/<STEP>/execution.json`。
- 本家stdout/stderrおよび本家のbook別log: 同directory。
- 元の本家出力: `*.omr`、展開内容 `omr-extracted/book.xml` と `sheet#1/sheet#1.xml`、`GRAY.png` / `BINARY.png`。
- 原RGBAのGRID: `runs/laura-original-rgba/GRID/summary.json`。native側比較にはgray入力の同ファイルも利用可能。
- 版とnative engine: `environment/version.log`。依存jar全件のSHAと取得情報: `environment/inventory.json`。

```sh
python3 run_oracle.py inputs/laura-original-rgba.png inputs/laura-gray.png inputs/synthetic-system-bars.png
python3 build_reference.py
```

各targetを新しい出力directoryで元画像から実行するため、LOAD画像が後続stageによって `.omr` から削除されても元stage成果物が残る。`-batch -step <STEP> -save -output <DIR> -- <IMAGE>` が公式CLIへの実際の指定。GUIを起動せず、OCR以降のstepを指定していない。

## 比較の契約

原RGBAとgray入力は本家LOAD後の画素もBINARY画素も**完全一致**した。このgray画素は既存PGMの元画素とも完全一致する。

- LOAD raw gray SHA-256: `1409021704fc10991babd90335ccb89a241cd73059a497dadf21a5b54fa234cb`
- BINARY raw 8-bit画素SHA-256: `a217f4ccc273afe4a7a86be2ef27504a953af5b2bb6e8d9e6a9e9582cd4a5126`
- BINARY前景: 189,777画素。
- SCALE: line `(1,2,3)`、interline `(11,11,12)`、beam `6`。

Staff形状は**本家が永続化したStaffLineの点列**である。`StaffLine` はfilament点から構築するときYへ0.5を加え、XMLのPoint2DAdapter/Double1Adapterは小数第1位まで保存する。比較時にfilamentの未補正座標や1e-9精度のin-memory値と混同しない。nativeも同じ座標系・1桁への丸めで照合するか、永続化量子化を考慮した許容差を明示する。未シリアライズの全section/filamentグラフをこのoracleが出力すると主張しない。

同じバージョン/入力/設定についてBINARYの画素、scaleの整数range、staff/line/system数と所属を厳密照合し、staffの点列・長さ・太さをこの保存精度で照合するのが次の受入れ条件になる。これ自体は音符・リズム認識やMusicXML精度の証明ではない。

## 環境・取得量・制約

当初の既存アプリは5.5.3、既存Javaは21。5.11.0はJava 25が必要でGradle cacheもなかった。公式arm64 DMG **84,929,243 bytes** とマウントせず展開する公式7-Zip CLI約1.9MBのみ取得。合計約86.8MB。数GBの取得、新規認証、システムインストールは不要だった。

DMG SHAはGitHub release metadataのdigestと一致: `17491af8b6d40153b031dd1f0815e37213f0999ef23f10586af46706e59b2eb6`。実行versionも **5.11.0 / 9e1e55cd2746037d059345881c53e6a6754bffbd**。同梱Zulu Java **25.0.3+9-LTS**、JavaCPP **1.5.13**、Leptonica **1.87.0**、Tesseract **5.5.2**。build/Gradle/ソース改変は不要だった。

DMGの通常マウントは環境で失敗し、7-Zipの展開で代替した。7-ZipはApplicationsリンクと36個のJavaライセンス文書への相対リンクを無視した（ログ保持）。jar/runtime実体は正常に読み込めた。元DMGを完全な状態で保持している。配布物やライセンスの再公開はしていない。

既存ユーザーのAudiveris設定には履歴・既定folder・releaseチェック日時のみがあり、解析定数のoverrideはなかった。履歴の個人情報は成果物へ複写していない。batch定数永続化をfalseとし、出力はworkspace明示。native cache/temp/prefsもworkspace指定。公式起動処理は既存OCR言語を列挙するが、この試験にOCR結果は使用しない。Java 25のnative-access warningはログに残したがGRIDは成功した。

CC0入力は `inputs/CC0-LICENSE.txt` と `inputs/laura-source-manifest.json` を保持。既存rendererがPNG保存後exit -6だった来歴も変更していない。今回の本家Java処理はexit 0で完了した。180 DPIの1ページのみであり、300 DPIや広範な実スキャンの保証ではない。

## 一次資料

- [公式CLI: batch/step/save](https://audiveris.github.io/audiveris/_pages/guides/advanced/cli/)
- [公式5.11.0配布](https://github.com/Audiveris/audiveris/releases/tag/5.11.0)
- [固定commitのStaffLine](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/sheet/StaffLine.java)
- [固定commitのJaxb精度](https://github.com/Audiveris/audiveris/blob/9e1e55cd2746037d059345881c53e6a6754bffbd/app/src/main/java/org/audiveris/omr/util/Jaxb.java)

同じ固定sourceの参照用コピーを `source/` に保存。実行したのはそれらを再構成したstubではなく公式配布jarである。
