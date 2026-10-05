// SPDX-License-Identifier: AGPL-3.0-or-later
// Generic original Bravura metrics; combo time bounds use NumDen dimensions.
import java.util.*;import java.nio.file.*;import java.awt.*;import java.awt.font.TextLayout;import java.awt.geom.*;
public class TimeFontMetricsProbe {
 public static void main(String[]args)throws Exception{
  Class<?> mf=Class.forName("org.audiveris.omr.ui.symbol.MusicFont"),fc=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily"),sc=Class.forName("org.audiveris.omr.glyph.Shape"),ss=Class.forName("org.audiveris.omr.glyph.ShapeSet");mf.getMethod("populateAllSymbols").invoke(null);Object family=fc.getField("Bravura").get(null);Set<Object> shapes=new LinkedHashSet<>();Collection<?> whole=(Collection<?>)ss.getField("WholeTimes").get(null),single=(Collection<?>)ss.getField("SingleWholeTimes").get(null);shapes.addAll(whole);shapes.addAll((Collection<?>)ss.getField("PartialTimes").get(null));java.util.List<String> entries=new ArrayList<>();
  for(Object shape:shapes)for(int il=1;il<=128;il++){
   Object fs=sc.getMethod("getFontSymbolByInterline",fc,int.class).invoke(shape,family,il);TextLayout layout=(TextLayout)GridProbe.call(fs,"getLayout");boolean combo=whole.contains(shape)&&!single.contains(shape);String extra;
   if(combo){
    Object inter=Class.forName("org.audiveris.omr.sig.inter.TimeWholeInter").getConstructor(Class.forName("org.audiveris.omr.glyph.Glyph"),sc,Double.class).newInstance(null,shape,1.0);int num=(Integer)GridProbe.call(inter,"getNumerator"),den=(Integer)GridProbe.call(inter,"getDenominator");Object font=mf.getMethod("getBaseFont",fc,int.class).invoke(null,family,il);Object symbol=Class.forName("org.audiveris.omr.ui.symbol.NumDenSymbol").getConstructor(sc,fc,int.class,int.class).newInstance(shape,family,num,den);Dimension dim=(Dimension)GridProbe.call(symbol,"getDimension",font);GridProbe.call(inter,"setBounds",new Rectangle(1000,1000,100,100));Rectangle original=(Rectangle)GridProbe.call(inter,"getSymbolBounds",il);if(!original.equals(new Rectangle(1050-dim.width/2,1050-dim.height/2,dim.width,dim.height)))throw new IllegalStateException("Combo bounds mismatch");
    extra="\"bounds_method\":\"numden_dimension_integer_half\",\"layout_is_null\":"+(layout==null)+",\"dimension_width\":"+dim.width+",\"dimension_height\":"+dim.height+",\"numerator\":"+num+",\"denominator\":"+den+",\"original_bounds_control\":["+original.x+","+original.y+","+original.width+","+original.height+"]";
   }else{
    if(layout==null)throw new IllegalStateException("Missing single layout: "+shape+" interline "+il);Rectangle2D r=layout.getBounds();if(r.getWidth()<=0||r.getHeight()<=0)throw new IllegalStateException("Nonpositive layout: "+shape+" interline "+il);
    extra="\"bounds_method\":\"layout_rint_half\",\"layout_x\":"+r.getX()+",\"layout_y\":"+r.getY()+",\"layout_width\":"+r.getWidth()+",\"layout_height\":"+r.getHeight();
   }
   entries.add("{\"shape\":\""+shape+"\",\"interline\":"+il+","+extra+"}");
  }
  Files.writeString(Path.of(args[0]),"{\"family\":\"Bravura\",\"upstream_commit\":\"9e1e55cd2746037d059345881c53e6a6754bffbd\",\"jar_sha256\":\"9b35257edea808ec80a8bcd48882d174b107152d9640b5ca33e1365593cff2a5\",\"font_resource\":\"res/Bravura.otf\",\"font_sha256\":\"dca2d90c88437a701b1c2e71fa54e76f9fa41d7deee935d74dc871ea66ecfdd2\",\"combo_control_input_bounds\":[1000,1000,100,100],\"entries\":["+String.join(",",entries)+"]}\n");System.exit(0);
 }
}
