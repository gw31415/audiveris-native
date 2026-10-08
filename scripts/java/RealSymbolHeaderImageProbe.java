// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;
import java.util.*;
import java.awt.Color;
import java.awt.Graphics2D;
import java.awt.Rectangle;
import java.awt.image.*;
import java.security.MessageDigest;
public class RealSymbolHeaderImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealSymbolHeaderImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 public void process(){try {
  Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("CHORDS").get(null),false);Object sheet=call(stub,"getSheet");
  int width=((Number)call(sheet,"getWidth")).intValue(),height=((Number)call(sheet,"getHeight")).intValue();BufferedImage mask=new BufferedImage(width,height,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=mask.createGraphics();g.setColor(Color.WHITE);List<String> rows=new ArrayList<>();
  Object buffer=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class).newInstance(width,height);var ctor=c("sheet.symbol.SymbolsFilter$SymbolsCleaner").getDeclaredConstructors()[0];ctor.setAccessible(true);Object cleaner=ctor.newInstance(buffer,g,sheet);var constantsField=c("sheet.symbol.SymbolsFilter").getDeclaredField("constants");constantsField.setAccessible(true);Object constants=constantsField.get(null);var marginField=constants.getClass().getDeclaredField("staffVerticalMargin");marginField.setAccessible(true);Object margin=marginField.get(constants);
  for(Object sys:(List<?>)call(sheet,"getSystems"))call(cleaner,"eraseStavesHeader",sys,margin);
  for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object staff:(List<?>)call(sys,"getStaves")){
   int left=((Number)call(staff,"getHeaderStart")).intValue(),right=((Number)call(staff,"getHeaderStop")).intValue(),il=((Number)call(staff,"getSpecificInterline")).intValue(),dy=(int)Math.rint(il*.5);
   int top=((Number)call(call(staff,"getFirstLine"),"yAt",right)).intValue()-dy,bot=((Number)call(call(staff,"getLastLine"),"yAt",right)).intValue()+dy;
   Rectangle r=new Rectangle(left,top,right-left+1,bot-top+1);rows.add("{\"system\":"+call(sys,"getId")+",\"staff\":"+call(staff,"getId")+",\"start\":"+left+",\"stop\":"+right+",\"interline\":"+il+",\"bounds\":"+rect(r)+"}");
  }g.dispose();byte[] pixels=((DataBufferByte)mask.getRaster().getDataBuffer()).getData();String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(pixels));
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"normal unchanged image CHORDS; PageCleaner.eraseStavesHeader rectangle at right ordinate and Java fillRect pixels; no native outputs\",\"width\":"+width+",\"height\":"+height+",\"rectangles\":["+String.join(",",rows)+"],\"mask_sha256\":\""+sha+"\"}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
