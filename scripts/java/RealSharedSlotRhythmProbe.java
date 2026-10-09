// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealSharedSlotRhythmProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealSharedSlotRhythmProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static String observed(Object sheet)throws Exception {
 List<String> systems=new ArrayList<>();for(Object sys:(List<?>)call(sheet,"getSystems")){
 List<String> chords=new ArrayList<>();
 for(Object part:(List<?>)call(sys,"getParts"))for(Object measure:(List<?>)call(part,"getMeasures"))for(Object chord:(Collection<?>)call(measure,"getHeadChords")){
 Object staff=call(chord,"getStaff");Object voice=call(chord,"getVoice"),slot=call(chord,"getSlot");boolean actualSlot=false;for(Object held:(List<?>)call(call(chord,"getMeasure"),"getStack").getClass().getMethod("getSlots").invoke(call(call(chord,"getMeasure"),"getStack")))if(held==slot)actualSlot=true;
 chords.add("{\"same_actual_stack_slot\":"+actualSlot+",\"bounds\":"+rect(call(chord,"getBounds"))+",\"part\":"+call(part,"getId")+",\"measure_index\":"+(((List<?>)call(part,"getMeasures")).indexOf(measure)+1)+",\"staff\":"+(staff==null?"null":call(staff,"getId"))+",\"voice\":"+(voice==null?"null":call(voice,"getId"))+",\"slot\":"+(slot==null?"null":call(slot,"getId"))+",\"time_offset\":\""+call(chord,"getTimeOffset")+"\",\"beam_count\":"+((Collection<?>)call(chord,"getBeams")).size()+",\"flags_number\":"+call(chord,"getFlagsNumber")+",\"dots_number\":"+call(chord,"getDotsNumber")+",\"duration_sans_dot_or_tuplet\":\""+call(chord,"getDurationSansDotOrTuplet")+"\",\"duration\":\""+call(chord,"getDuration")+"\"}");}
 systems.add("{\"system\":"+call(sys,"getId")+",\"chords\":["+String.join(",",chords)+"]}");}return "["+String.join(",",systems)+"]";
 }
 public void process(){try {Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("RHYTHMS").get(null),false);Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged original image ordinary RHYTHMS actual Chord beam/flag/dot duration and Voice/Slot clocks, no native input\",\"systems\":"+observed(call(stub,"getSheet"))+"}\n");}catch(Exception e){throw new RuntimeException(e);}}
}
