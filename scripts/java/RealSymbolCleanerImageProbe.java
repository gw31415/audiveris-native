// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;
import java.util.*;
import java.awt.Graphics2D;
import java.awt.image.*;
import java.security.MessageDigest;
public class RealSymbolCleanerImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealSymbolCleanerImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 public void process(){try {
  Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("CHORDS").get(null),false);Object sheet=call(stub,"getSheet");Object raw=call(call(sheet,"getPicture"),"getSource",c("sheet.Picture$SourceKey").getField("NO_STAFF").get(null));BufferedImage image=(BufferedImage)call(raw,"getBufferedImage");Object buffer=Class.forName("ij.process.ByteProcessor").getConstructor(java.awt.Image.class).newInstance(image);Graphics2D g=image.createGraphics();var ctor=c("sheet.symbol.SymbolsFilter$SymbolsCleaner").getDeclaredConstructors()[0];ctor.setAccessible(true);Object cleaner=ctor.newInstance(buffer,g,sheet);List<String> systems=new ArrayList<>();Map<Object,List<?>> optionals=new LinkedHashMap<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){List<String> rows=new ArrayList<>();for(Object n:(Set<?>)call(call(sys,"getSig"),"vertexSet")){if((Boolean)call(n,"isRemoved"))continue;Object ensemble=call(n,"getEnsemble");String kind=n.getClass().getSimpleName();rows.add("{\"kind\":\""+kind+"\",\"bounds\":"+rect(call(n,"getBounds"))+",\"frozen\":"+call(n,"isFrozen")+",\"grade\":"+call(n,"getGrade")+",\"best_grade\":"+call(n,"getBestGrade")+",\"ensemble\":"+(ensemble==null?"null":"\""+ensemble.getClass().getSimpleName()+"\"")+",\"can_hide\":"+call(cleaner,"canHide",n)+"}");}systems.add("{\"system\":"+call(sys,"getId")+",\"inters\":["+String.join(",",rows)+"]}");}
  call(cleaner,"eraseInters",optionals);g.dispose();call(buffer,"threshold",127);List<String> optionalRows=new ArrayList<>();for(var e:optionals.entrySet()){List<String> glyphs=new ArrayList<>();for(Object glyph:e.getValue())glyphs.add(glyph(glyph));optionalRows.add("{\"system\":"+call(e.getKey(),"getId")+",\"glyphs\":["+String.join(",",glyphs)+"]}");}byte[] pixels=(byte[])call(buffer,"getPixels");String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(pixels));Files.write(Path.of(System.getProperty("native.probe.output")+".pixels"),pixels);
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"normal unchanged image CHORDS, actual SymbolsCleaner eraseInters partition and resulting NO_STAFF pixel mask; no native outputs\",\"systems\":["+String.join(",",systems)+"],\"optionals\":["+String.join(",",optionalRows)+"],\"pixel_sha256\":\""+sha+"\"}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
