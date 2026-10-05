// SPDX-License-Identifier: AGPL-3.0-or-later
// Observe actual original linking/finalize/reduction, without native outputs.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;
public class StemImageScheduleProbe extends StemImageHeadExpansionProbe {
 private final Object book;
 public StemImageScheduleProbe(Book b, SortedSet<Integer> ids) {super(b,ids);book=b;}
 static String state(Object sys) throws Exception {
  Object sig=call(sys,"getSig");List<String>heads=new ArrayList<>(),stems=new ArrayList<>(),edges=new ArrayList<>();
  for(Object v:(Collection<?>)call(sig,"vertexSet")) {
   if(v.getClass().getSimpleName().equals("HeadInter")) heads.add("{\"bounds\":"+rect(call(v,"getBounds"))+",\"shape\":\""+call(v,"getShape")+"\",\"grade\":"+call(v,"getGrade")+",\"abnormal\":"+call(v,"isAbnormal")+"}");
   if(v.getClass().getSimpleName().equals("StemInter")) stems.add(created(v));
  }
  for(Object e:(Collection<?>)call(sig,"edgeSet"))if(e.getClass().getSimpleName().equals("HeadStemRelation")) {
   Object h=call(sig,"getEdgeSource",e),s=call(sig,"getEdgeTarget",e);
   edges.add("{\"head_bounds\":"+rect(call(h,"getBounds"))+",\"shape\":\""+call(h,"getShape")+"\",\"stem\":"+created(s)+",\"relation\":"+relation(e)+"}");
  }
  return "{\"heads\":["+String.join(",",heads)+"],\"stems\":["+String.join(",",stems)+"],\"head_stem_relations\":["+String.join(",",edges)+"]}";
 }
 public void process(){try {
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<String>systems=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")) {
   Object retr=inspect(sys,sheet);String before=state(sys);
   call(retr,"linkStems");String linked=state(sys);
   call(retr,"finalizeStems");String finalized=state(sys);
   Object reducer=Class.forName("org.audiveris.omr.sig.SigReducer").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo"),boolean.class).newInstance(sys,true);
   call(reducer,"reduceFoundations");String reduced=state(sys);
   systems.add("{\"system\":"+call(sys,"getId")+",\"before\":"+before+",\"linked\":"+linked+",\"finalized\":"+finalized+",\"reduced\":"+reduced+"}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"scope\":\"original_XML_loaded_actual_linkStems_finalizeStems_reduceFoundations\",\"systems\":["+String.join(",",systems)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
