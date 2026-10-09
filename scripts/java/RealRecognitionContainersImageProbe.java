// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealRecognitionContainersImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealRecognitionContainersImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static Collection<?> staffNotes(Object staff)throws Exception {var f=c("sheet.Staff").getDeclaredField("notes");f.setAccessible(true);Object notes=f.get(staff);return notes==null?List.of():(Collection<?>)notes;}
 static String snapshot(Object sheet)throws Exception {
 List<String> systems=new ArrayList<>();for(Object sys:(List<?>)call(sheet,"getSystems")){
 List<String> notes=new ArrayList<>(),chords=new ArrayList<>();Object sig=call(sys,"getSig");
 for(Object staff:(List<?>)call(sys,"getStaves")){List<String> rows=new ArrayList<>();for(Object note:staffNotes(staff))rows.add("{\"kind\":\""+note.getClass().getSimpleName()+"\",\"bounds\":"+rect(call(note,"getBounds"))+"}");notes.add("{\"staff\":"+call(staff,"getId")+",\"notes\":["+String.join(",",rows)+"]}");}
 for(Object part:(List<?>)call(sys,"getParts"))for(Object measure:(List<?>)call(part,"getMeasures"))for(String getter:new String[]{"getHeadChords","getRestChords"})for(Object chord:(Collection<?>)call(measure,getter)){
 Object staff=call(chord,"getStaff");List<String> members=new ArrayList<>();for(Object member:(List<?>)call(chord,"getNotes"))members.add("{\"kind\":\""+member.getClass().getSimpleName()+"\",\"bounds\":"+rect(call(member,"getBounds"))+",\"staff\":"+call(call(member,"getStaff"),"getId")+"}");
 chords.add("{\"kind\":\""+chord.getClass().getSimpleName()+"\",\"bounds\":"+rect(call(chord,"getBounds"))+",\"part\":"+call(part,"getId")+",\"measure_index\":"+(((List<?>)call(part,"getMeasures")).indexOf(measure)+1)+",\"staff\":"+(staff==null?"null":call(staff,"getId"))+",\"same_actual_Part_Measure_object\":"+(call(chord,"getPart")==part&&call(chord,"getMeasure")==measure)+",\"members\":["+String.join(",",members)+"]}");}
 systems.add("{\"system\":"+call(sys,"getId")+",\"staff_notes\":["+String.join(",",notes)+"],\"chords\":["+String.join(",",chords)+"]}");}return "["+String.join(",",systems)+"]";
 }
 public void process(){try {Object stub=((List<?>)call(actual,"getStubs")).get(0);List<String> stages=new ArrayList<>();for(String name:new String[]{"CHORDS","SYMBOLS","LINKS"}){call(stub,"reachStep",c("step.OmrStep").getField(name).get(null),false);Object sheet=call(stub,"getSheet");stages.add("{\"stage\":\""+name+"\",\"systems\":"+snapshot(sheet)+"}");}Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged original image normal reachStep CHORDS/SYMBOLS/LINKS, actual Staff notes and Part Measure Chord/member object ownership, no native input\",\"stages\":["+String.join(",",stages)+"]}\n");}catch(Exception e){throw new RuntimeException(e);}}
}
