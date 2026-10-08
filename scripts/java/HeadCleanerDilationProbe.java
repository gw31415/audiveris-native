// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.Rectangle;import java.lang.reflect.*;import java.nio.file.*;import java.util.*;
public class HeadCleanerDilationProbe {
 static Class<?> c(String n)throws Exception{return Class.forName("org.audiveris.omr."+n);}
 static Object call(Object o,String n,Object...a)throws Exception{for(Method m:o.getClass().getMethods())if(m.getName().equals(n)&&m.getParameterCount()==a.length)try{return m.invoke(o,a);}catch(IllegalArgumentException e){}throw new NoSuchMethodException(n);}
 public static void main(String[] args)throws Exception{
 c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
 Object factory=c("image.TemplateFactory").getMethod("getInstance").invoke(null),family=c("ui.symbol.MusicFamily").getField("Bravura").get(null),shape=c("glyph.Shape").getField("NOTEHEAD_BLACK").get(null);List<String> cases=new ArrayList<>();
 for(int size:new int[]{32,40,44,47,51,64}){Object tpl=call(call(factory,"getCatalog",family,size),"getTemplate",shape);Object seed=null;for(Object kp:(List<?>)call(tpl,"getKeyPoints"))if(kp.getClass().getField("d").getDouble(kp)==0){seed=kp;break;}if(seed==null)throw new IllegalStateException("foreground point required");int px=seed.getClass().getField("x").getInt(seed),py=seed.getClass().getField("y").getInt(seed);byte[] pixels=new byte[128*128];Arrays.fill(pixels,(byte)255);pixels[(32+py)*128+32+px]=0;Object buffer=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class,byte[].class).newInstance(128,128,pixels);List<?> foreground=(List<?>)call(tpl,"getForegroundPixels",new Rectangle(32,32,(Integer)call(tpl,"getWidth"),(Integer)call(tpl,"getHeight")),buffer,true);int left=128,top=128,right=-128,bottom=-128;for(Object value:foreground){java.awt.Point p=(java.awt.Point)value;left=Math.min(left,p.x);top=Math.min(top,p.y);right=Math.max(right,p.x);bottom=Math.max(bottom,p.y);}cases.add("{\"point_size\":"+size+",\"single_foreground_point\":["+px+","+py+"],\"dilated_bounds\":["+left+","+top+","+(right-left+1)+","+(bottom-top+1)+"],\"dilated_pixels\":"+foreground.size()+"}");}
 Files.writeString(Path.of(args[0]),"{\"scope\":\"six declared single foreground pixel controls through actual original Template.getForegroundPixels and ByteProcessor.dilate; not real-score quality\",\"native_outputs_used\":false,\"cases\":["+String.join(",",cases)+"]}\n");
 }
}
