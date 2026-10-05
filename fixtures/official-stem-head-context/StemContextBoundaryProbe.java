// SPDX-License-Identifier: AGPL-3.0-or-later
// Original SIGraph contextual grades at the actual linkStems boundary.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;
public class StemContextBoundaryProbe extends StemReductionTraceProbe {
 private final Object book;
 public StemContextBoundaryProbe(Book b, SortedSet<Integer> ids){super(b,ids);book=b;}
 public void process(){try{
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<String> systems=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){
   Object retr=inspect(sys,sheet);call(retr,"linkStems");call(call(sys,"getSig"),"contextualize");
   systems.add("{\"system\":"+call(sys,"getId")+",\"detail\":"+detailed(sys)+"}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"scope\":\"original_XML_loaded_compute_contextual_grades_after_linkStems_before_finalization_and_reduction\",\"systems\":["+String.join(",",systems)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
