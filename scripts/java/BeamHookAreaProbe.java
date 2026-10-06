// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.*;import java.awt.image.*;import java.awt.geom.*;import java.nio.file.*;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class BeamHookAreaProbe extends BeamOriginDependencyProbe {
 static Object rows(Area area){BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=p.createGraphics();g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_OFF);g.setColor(Color.WHITE);g.setStroke(new BasicStroke(1.1f,BasicStroke.CAP_SQUARE,BasicStroke.JOIN_MITER));g.fill(area);g.draw(area);g.dispose();java.util.List<Object>rows=new ArrayList<>();for(int y=0;y<420;y++){int x0=1280,x1=-1;for(int x=0;x<1280;x++)if(p.getRaster().getSample(x,y,0)==255){x0=Math.min(x0,x);x1=Math.max(x1,x);}if(x1>=0)rows.add(new int[]{y,x0,x1+1});}return rows;}
 public static void main(String[]args)throws Exception{
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((java.util.List<?>)call(book,"getStubs")).get(0),"getSheet");java.util.List<Object>out=new ArrayList<>();
  for(Object sys:(java.util.List<?>)call(sheet,"getSystems"))for(Object b:(Collection<?>)call(call(sys,"getSig"),"vertexSet"))if(b.getClass().getSimpleName().equals("BeamInter")||b.getClass().getSimpleName().equals("BeamHookInter"))out.add(obj("scope","actual HEADS-restored beam or hook","kind",b.getClass().getSimpleName(),"median",line(call(b,"getMedian")),"height",call(b,"getHeight"),"white_rows",rows((Area)call(b,"getArea"))));
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original PageCleaner BasicStroke 1.1/AA-off fill+draw on five actual restored beam/hook areas; separate from full-image recognition parity","native_outputs_used",false,"cases",out));System.exit(0);
 }
}
