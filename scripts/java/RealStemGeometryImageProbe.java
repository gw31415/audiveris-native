// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;
import java.util.*;
import java.awt.geom.Line2D;
public class RealStemGeometryImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 private final IdentityHashMap<Object,Integer> identities=new IdentityHashMap<>();
 public RealStemGeometryImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 String snapshot(Object sheet,String phase)throws Exception {
  List<String> systems=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){List<String> rows=new ArrayList<>();for(Object stem:(List<?>)call(call(sys,"getSig"),"inters",c("sig.inter.StemInter"))){Object glyph=call(stem,"getGlyph");Line2D line=(Line2D)call(stem,"getMedian");List<String> heads=new ArrayList<>();for(Object h:(Collection<?>)call(stem,"getHeads"))heads.add("{\"bounds\":"+rect(call(h,"getBounds"))+",\"staff\":"+call(call(h,"getStaff"),"getId")+"}");if(!identities.containsKey(stem))identities.put(stem,identities.size()+1);
    rows.add("{\"identity\":"+identities.get(stem)+",\"bounds\":"+rect(call(stem,"getBounds"))+",\"glyph_bounds\":"+rect(call(glyph,"getBounds"))+",\"glyph_weight\":"+call(glyph,"getWeight")+",\"median\":["+line.getX1()+","+line.getY1()+","+line.getX2()+","+line.getY2()+"],\"width\":"+call(stem,"getWidth")+",\"heads\":["+String.join(",",heads)+"]}");}
   systems.add("{\"system\":"+call(sys,"getId")+",\"stems\":["+String.join(",",rows)+"]}");}
  return "{\"phase\":\""+phase+"\",\"systems\":["+String.join(",",systems)+"]}";
 }
 public void process(){try {Object stub=((List<?>)call(actual,"getStubs")).get(0);List<String> frames=new ArrayList<>();for(String step:List.of("STEMS","CHORDS")){call(stub,"reachStep",c("step.OmrStep").getField(step).get(null),false);frames.add(snapshot(call(stub,"getSheet"),step));}Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"unchanged image original normal STEMS then CHORDS in memory; actual Stem/glyph constructor geometry and held object identities, no native outputs\",\"frames\":["+String.join(",",frames)+"]}\n");}catch(Exception e){throw new RuntimeException(e);}}
}
