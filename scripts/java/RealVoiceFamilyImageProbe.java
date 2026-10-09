// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealVoiceFamilyImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealVoiceFamilyImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 public void process(){try {Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("LINKS").get(null),false);Object sheet=call(stub,"getSheet");List<String> rows=new ArrayList<>();
 for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object part:(List<?>)call(sys,"getParts"))for(Object measure:(List<?>)call(part,"getMeasures"))for(Object chord:(Collection<?>)call(measure,"getHeadChords")){
 Object staff=call(chord,"getTopStaff"),kind=call(measure,"inferVoiceKind",chord);rows.add("{\"system\":"+call(sys,"getId")+",\"part\":"+call(part,"getId")+",\"measure_index\":"+(((List<?>)call(part,"getMeasures")).indexOf(measure)+1)+",\"bounds\":"+rect(call(chord,"getBounds"))+",\"merged_part\":"+call(part,"isMerged")+",\"top_staff\":"+call(staff,"getId")+",\"same_actual_Part_Staff\":"+(call(staff,"getPart")==part)+",\"kind\":\""+kind+"\",\"kind_ordinal\":"+((Enum<?>)kind).ordinal()+",\"id_offset\":"+call(kind,"idOffset")+",\"next_voice_id\":"+call(measure,"generateVoiceId",kind)+",\"existing_voices\":"+((List<?>)call(measure,"getVoices")).size()+"}");}
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged original LINKS actual Part/Measure/Chord top Staff, inferVoiceKind and next ID reads before any rhythm Voice constructor, no native input\",\"chords\":["+String.join(",",rows)+"]}\n");}catch(Exception e){throw new RuntimeException(e);}}
}
