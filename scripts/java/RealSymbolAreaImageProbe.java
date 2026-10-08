// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;
import java.util.*;
import java.awt.Color;
import java.awt.Graphics2D;
import java.awt.geom.*;
import java.awt.image.*;
public class RealSymbolAreaImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealSymbolAreaImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static String line(Object value){Line2D l=(Line2D)value;return "["+l.getX1()+","+l.getY1()+","+l.getX2()+","+l.getY2()+"]";}
 public void process(){try {
  Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("CHORDS").get(null),false);Object sheet=call(stub,"getSheet");int width=((Number)call(sheet,"getWidth")).intValue(),height=((Number)call(sheet,"getHeight")).intValue();BufferedImage mask=new BufferedImage(width,height,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=mask.createGraphics();Object buffer=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class).newInstance(width,height);var ctor=c("sheet.symbol.SymbolsFilter$SymbolsCleaner").getDeclaredConstructors()[0];ctor.setAccessible(true);Object cleaner=ctor.newInstance(buffer,g,sheet);List<String> systems=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){List<String> rows=new ArrayList<>();Object sig=call(sys,"getSig");for(String type:List.of("BarlineInter","BarConnectorInter","AbstractBeamInter"))for(Object n:(List<?>)call(sig,"inters",c("sig.inter."+type))){g.setColor(Color.BLACK);g.fillRect(0,0,width,height);g.setColor(Color.WHITE);call(n,"accept",cleaner);Area area=(Area)call(n,"getArea");java.awt.Rectangle box=area.getBounds();box.grow(3,3);List<String> raster=new ArrayList<>();for(int y=Math.max(0,box.y);y<Math.min(height,box.y+box.height);y++){int left=-1,right=-1;for(int x=Math.max(0,box.x);x<Math.min(width,box.x+box.width);x++)if(mask.getRaster().getSample(x,y,0)!=0){if(left<0)left=x;right=x+1;}if(left>=0)raster.add("["+y+","+left+","+right+"]");}
   String dimension=type.equals("AbstractBeamInter")?"\"height\":"+call(n,"getHeight"):"\"width\":"+call(n,"getWidth");rows.add("{\"kind\":\""+n.getClass().getSimpleName()+"\",\"bounds\":"+rect(call(n,"getBounds"))+",\"median\":"+line(call(n,"getMedian"))+","+dimension+",\"white_rows\":["+String.join(",",raster)+"]}");
  }systems.add("{\"system\":"+call(sys,"getId")+",\"inters\":["+String.join(",",rows)+"]}");}g.dispose();Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"original normal unchanged image CHORDS; actual Inter PageCleaner area fill/stroke on isolated canvas, no native outputs\",\"systems\":["+String.join(",",systems)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
