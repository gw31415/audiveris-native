// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;
import java.util.*;
import java.awt.geom.Point2D;
public class RealHeadChordsImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealHeadChordsImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static String point(Object p){return p==null?"null":"["+((Point2D)p).getX()+","+((Point2D)p).getY()+"]";}
 static String note(Object n)throws Exception{return "{\"bounds\":"+rect(call(n,"getBounds"))+",\"shape\":\""+call(n,"getShape")+"\",\"staff\":"+call(call(n,"getStaff"),"getId")+",\"pitch\":"+call(n,"getIntegerPitch")+"}";}
 public void process(){try {
  Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("CHORDS").get(null),false);Object sheet=call(stub,"getSheet");List<String> systems=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){List<String> rows=new ArrayList<>();List<?> actualParts=(List<?>)call(sys,"getParts");Object sig=call(sys,"getSig");
   for(Object chord:(List<?>)call(sig,"inters",c("sig.inter.HeadChordInter"))){Object part=call(chord,"getPart"),staff=call(chord,"getStaff"),measure=call(chord,"getMeasure"),stem=call(chord,"getStem");if(!actualParts.contains(part)||measure==null||call(measure,"getPart")!=part)throw new IllegalStateException("Chord actual Part/Measure required");List<String> members=new ArrayList<>();for(Object n:(List<?>)call(chord,"getMembers"))members.add(note(n));List<?> measures=(List<?>)call(part,"getMeasures"),stacks=(List<?>)call(sys,"getStacks");if(!((Collection<?>)call(measure,"getHeadChords")).contains(chord))throw new IllegalStateException("Actual chord not registered in Measure");
    rows.add("{\"bounds\":"+rect(call(chord,"getBounds"))+",\"staff\":"+(staff==null?"null":call(staff,"getId"))+",\"part\":"+call(part,"getId")+",\"measure_index\":"+(measures.indexOf(measure)+1)+",\"stack_index\":"+(stacks.indexOf(call(measure,"getStack"))+1)+",\"stem_bounds\":"+(stem==null?"null":rect(call(stem,"getBounds")))+",\"head_location\":"+point(call(chord,"getHeadLocation"))+",\"tail_location\":"+point(call(chord,"getTailLocation"))+",\"members\":["+String.join(",",members)+"],\"same_actual_Part_Measure_object\":true}");
   }
   systems.add("{\"system\":"+call(sys,"getId")+",\"chords\":["+String.join(",",rows)+"]}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"unchanged real image through normal original reachStep CHORDS in memory before intermediate serialization; actual Part/Measure/Stack/Heads/stem refs, no native outputs\",\"systems\":["+String.join(",",systems)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
