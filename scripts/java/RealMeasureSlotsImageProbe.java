// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealMeasureSlotsImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealMeasureSlotsImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 private static String slots(List<?> slots)throws Exception{List<String> rows=new ArrayList<>();for(Object slot:slots){List<String> chords=new ArrayList<>();for(Object ch:(List<?>)call(slot,"getChords"))chords.add(rect(call(ch,"getBounds")));rows.add("{\"id\":"+call(slot,"getId")+",\"chord_bounds\":["+String.join(",",chords)+"]}");}return "["+String.join(",",rows)+"]";}
 public void process(){try{Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("LINKS").get(null),false);Object sheet=call(stub,"getSheet");List<String> rows=new ArrayList<>();
 for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object part:(List<?>)call(sys,"getParts"))for(Object measure:(List<?>)call(part,"getMeasures")){
 Object stack=call(measure,"getStack");List<String> offsets=new ArrayList<>();for(Object ch:(Collection<?>)call(measure,"getStandardChords"))offsets.add("{\"bounds\":"+rect(call(ch,"getBounds"))+",\"x_offset\":"+call(stack,"getXOffset",call(ch,"getCenter"))+"}");
 List<?> narrow=(List<?>)call(make("sheet.rhythm.SlotsRetriever",measure,false),"buildSlots"),wide=(List<?>)call(make("sheet.rhythm.SlotsRetriever",measure,true),"buildSlots"),compounds=(List<?>)call(make("sheet.rhythm.MeasureRhythm",measure),"buildCompoundSlots",narrow,wide);List<String> cs=new ArrayList<>();for(Object compound:compounds){List<String> chords=new ArrayList<>(),members=new ArrayList<>();for(Object ch:(List<?>)call(compound,"getChords"))chords.add(rect(call(ch,"getBounds")));for(Object member:(List<?>)call(compound,"getMembers"))members.add(call(member,"getId").toString());cs.add("{\"id\":"+call(compound,"getId")+",\"narrow_slot_ids\":["+String.join(",",members)+"],\"chord_bounds\":["+String.join(",",chords)+"]}");}
 rows.add("{\"system\":"+call(sys,"getId")+",\"part\":"+call(part,"getId")+",\"measure_index\":"+(((List<?>)call(part,"getMeasures")).indexOf(measure)+1)+",\"offsets\":["+String.join(",",offsets)+"],\"narrow\":"+slots(narrow)+",\"wide\":"+slots(wide)+",\"compounds\":["+String.join(",",cs)+"]}");}
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged original LINKS actual Stack offsets and narrow/wide/compound retrieval before rhythm clocks, no native input\",\"measures\":["+String.join(",",rows)+"]}\n");}catch(Exception e){throw new RuntimeException(e);}}
}
