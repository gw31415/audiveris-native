// SPDX-License-Identifier: AGPL-3.0-or-later
// Generic original NOTEHEAD_BLACK TextLayout metrics and interpolation controls.
// No image fixtures or native output are inputs to this observer.
import java.util.*;
import java.nio.file.*;
import java.awt.font.TextLayout;
public class BlackHeadFontMetricsProbe {
 public static void main(String[] args)throws Exception {
  Class<?> mf=Class.forName("org.audiveris.omr.ui.symbol.MusicFont"),fc=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily"),sc=Class.forName("org.audiveris.omr.glyph.Shape");
  mf.getMethod("populateAllSymbols").invoke(null);
  Object family=fc.getField("Bravura").get(null),shape=sc.getField("NOTEHEAD_BLACK").get(null);
  List<String> widths=new ArrayList<>(),cases=new ArrayList<>();
  for(int size=0;size<=1024;size++) {
   Object font=mf.getConstructor(fc,int.class).newInstance(family,size);
   TextLayout layout=(TextLayout)mf.getMethod("layoutShapeByCode",sc).invoke(font,shape);
   widths.add(Double.toString(layout.getBounds().getWidth()));
  }
  Object font=mf.getConstructor(fc,int.class).newInstance(family,0);
  for(int i=1;i<=2048;i++) {double width=i/8.0;Object scale=mf.getMethod("buildMusicFontScale",double.class).invoke(font,width);cases.add("["+width+","+scale.getClass().getMethod("getPointSize").invoke(scale)+"]");}
  String metadata="\"family\":\"Bravura\",\"shape\":\"NOTEHEAD_BLACK\",\"point_size_min\":0,\"point_size_max\":1024,\"upstream_commit\":\"9e1e55cd2746037d059345881c53e6a6754bffbd\",\"jar_sha256\":\"9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5\",\"font_resource\":\"res/Bravura.otf\",\"font_sha256\":\"dca2d90c88437a701b1c2e71fa54e76f9fa41d7deee935d74dc871ea66ecfdd2\"";
  Files.writeString(Path.of(args[0]),"{"+metadata+",\"widths\":["+String.join(",",widths)+"],\"interpolation_controls\":["+String.join(",",cases)+"]}\n");
  System.exit(0);
 }
}
