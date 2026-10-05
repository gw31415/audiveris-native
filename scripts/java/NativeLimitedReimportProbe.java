// Development-only proxymusic reimport and fixed reference engraving.
// Actual native MusicXML is the only musical input; no recognition JSON/XML.
import java.util.*;import java.nio.file.*;import java.awt.*;import java.awt.image.*;
public class NativeLimitedReimportProbe {
 static Object call(Object o,String n,Object...a)throws Exception{return HeadActualPageRenderer.call(o,n,a);}
 static void staff(BufferedImage p,Graphics2D g,int top,boolean meter)throws Exception{
  for(int j=0;j<5;j++)g.fillRect(80,top+j*11,1120,2);for(int x:new int[]{80,710,1198})g.fillRect(x,top,2,46);
  HeadActualPageRenderer.stamp(p,HeadActualPageRenderer.symbol("Bravura","G_CLEF",51),100,top+22);
  if(meter)for(int y:new int[]{top+11,top+33})HeadActualPageRenderer.stamp(p,HeadActualPageRenderer.symbol("Bravura","TIME_FOUR",51),165,y);
 }
 public static void main(String[]a)throws Exception{
  Path out=Path.of(a[1]);Files.createDirectories(out);Object score;
  try(var in=Files.newInputStream(Path.of(a[0]))){score=Class.forName("org.audiveris.proxymusic.util.Marshalling").getMethod("unmarshal",java.io.InputStream.class).invoke(null,in);}
  java.util.List<?> parts=(java.util.List<?>)call(score,"getPart");if(parts.size()!=1)throw new IllegalStateException("one part");
  java.util.List<?> measures=(java.util.List<?>)call(parts.get(0),"getMeasure");
  call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  BufferedImage p=new BufferedImage(1280,measures.size()>2?760:420,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,p.getWidth(),p.getHeight());g.setColor(Color.BLACK);
  java.util.List<String> rows=new ArrayList<>();int mi=0,system=1,systemMeasure=0;staff(p,g,220,true);
  for(Object m:measures){mi++;boolean newSystem=false;
   java.util.List<?> content=(java.util.List<?>)call(m,"getNoteOrBackupOrForward");
   for(Object n:content)if(n.getClass().getSimpleName().equals("Print")&&call(n,"getNewSystem")!=null)newSystem=call(call(n,"getNewSystem"),"value").equals("yes");
   if(newSystem){system++;systemMeasure=0;staff(p,g,540,false);}systemMeasure++;int offset=0,top=system==1?220:540;
   for(Object n:content)if(n.getClass().getSimpleName().equals("Note")){
    Object pitch=call(n,"getPitch"),rest=call(n,"getRest");int duration=((java.math.BigDecimal)call(n,"getDuration")).intValueExact();String type=call(call(n,"getType"),"getValue").toString(),voice=call(n,"getVoice").toString();
    if(duration!=1&&duration!=2)throw new IllegalStateException("quarter/half renderer only");
    if(!type.equals(duration==1?"quarter":"half"))throw new IllegalStateException("duration/type mismatch");
    String step=pitch==null?null:call(pitch,"getStep").toString();Integer octave=pitch==null?null:(Integer)call(pitch,"getOctave");
    int pos=pitch==null?0:34-(7*octave+"CDEFGAB".indexOf(step));int x=(systemMeasure==1?270:750)+120*offset,cy=top+22+pos*11/2;
    String shape=rest!=null?(duration==1?"QUARTER_REST":"HALF_REST"):(duration==1?"NOTEHEAD_BLACK":"NOTEHEAD_VOID");if(shape.equals("HALF_REST"))cy=top+20;
    BufferedImage h=HeadActualPageRenderer.symbol("Bravura",shape,51);HeadActualPageRenderer.stamp(p,h,x,cy);if(rest==null)g.fillRect(x+h.getWidth()-(duration==2?6:2),cy-36,2,37);
    rows.add("{\"measure\":"+mi+",\"system\":"+system+",\"rest\":"+(rest!=null)+",\"step\":"+(step==null?"null":"\""+step+"\"")+",\"octave\":"+octave+",\"duration_quarters\":"+duration+",\"offset_quarters\":"+offset+",\"voice\":\""+voice+"\",\"type\":\""+type+"\"}");offset+=duration;
   }
   if(offset!=4)throw new IllegalStateException("complete 4/4 required");
  }
  g.dispose();javax.imageio.ImageIO.write(p,"PNG",out.resolve("reimport-render.png").toFile());Files.writeString(out.resolve("result.json"),"{\"original_proxymusic_reimported_actual_native_MusicXML\":true,\"fixed_reference_renderer_not_general_engraver\":true,\"events\":["+String.join(",",rows)+"]}\n");
 }
}
