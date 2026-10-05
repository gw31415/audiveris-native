// SPDX-License-Identifier: AGPL-3.0-or-later
import java.util.*;import java.nio.file.*;import java.awt.geom.Rectangle2D;
public class BracketFontProbe {
 @SuppressWarnings({"unchecked","rawtypes"})
 public static void main(String[]args)throws Exception{
  Class<?> mf=Class.forName("org.audiveris.omr.ui.symbol.MusicFont"),family=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily"),shape=Class.forName("org.audiveris.omr.glyph.Shape");
  mf.getMethod("populateAllSymbols").invoke(null);List<String>out=new ArrayList<>();
  for(int il:new int[]{11,16}){
   Object font=mf.getMethod("getBaseFont",family,int.class).invoke(null,Enum.valueOf((Class)family,"Bravura"),il);
   List<String>rects=new ArrayList<>();
   for(String name:List.of("BRACKET_UPPER_SERIF","BRACKET_LOWER_SERIF")){
    Object layout=mf.getMethod("layoutShapeByCode",shape).invoke(font,Enum.valueOf((Class)shape,name));Rectangle2D r=(Rectangle2D)layout.getClass().getMethod("getBounds").invoke(layout);
    rects.add("\""+name+"\":["+r.getX()+","+r.getY()+","+r.getWidth()+","+r.getHeight()+"]");
   }
   out.add("{\"interline\":"+il+",\"bounds\":{"+String.join(",",rects)+"}}");
  }
  Files.writeString(Path.of(args[0]),"["+String.join(",",out)+"]\n");System.exit(0);
 }
}
