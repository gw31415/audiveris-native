// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;
import java.util.*;
public class RealLedgerReductionImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealLedgerReductionImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static String frame(Object sys,List<Object> nodes,Map<Object,String> names)throws Exception{
  Object sig=call(sys,"getSig");List<String> maps=new ArrayList<>(),bodies=new ArrayList<>();for(Object staff:(List<?>)call(sys,"getStaves")){List<String> lines=new ArrayList<>();for(var e:((Map<?,?>)call(staff,"getLedgerMap")).entrySet()){List<String> ids=new ArrayList<>();for(Object n:(List<?>)e.getValue())ids.add("\""+names.get(n)+"\"");lines.add("{\"index\":"+e.getKey()+",\"members\":["+String.join(",",ids)+"]}");}maps.add("{\"staff\":"+call(staff,"getId")+",\"lines\":["+String.join(",",lines)+"]}");}
  for(Object n:nodes)bodies.add("{\"label\":\""+names.get(n)+"\",\"index\":"+call(n,"getIndex")+",\"owner_staff\":"+call(call(n,"getStaff"),"getId")+",\"removed\":"+call(n,"isRemoved")+",\"registered\":"+call(sig,"containsVertex",n)+"}");return "{\"maps\":["+String.join(",",maps)+"],\"bodies\":["+String.join(",",bodies)+"]}";
 }
 public void process(){try{
  Object sheet=call(((List<?>)call(actual,"getStubs")).get(0),"getSheet");for(String step:List.of("sheet.grid.GridStep","sheet.header.HeadersStep","sheet.stem.StemSeedsStep","sheet.beam.BeamsStep","sheet.ledger.LedgersStep","sheet.note.HeadsStep"))call(make(step),"doit",sheet);List<String> systems=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){List<Object> nodes=new ArrayList<>();Map<Object,String> names=new IdentityHashMap<>();for(Object staff:(List<?>)call(sys,"getStaves"))for(Object line:((Map<?,?>)call(staff,"getLedgerMap")).values())for(Object n:(List<?>)line)if(!names.containsKey(n)){names.put(n,"ledger"+nodes.size());nodes.add(n);}String before=frame(sys,nodes,names);Object reducer=make("sig.SigReducer",sys,false);Object shared=call(reducer,"fixAllSharedLedgers"),removed=call(reducer,"checkLedgers");String after=frame(sys,nodes,names);Object repeat=call(reducer,"checkLedgers");systems.add("{\"system\":"+call(sys,"getId")+",\"before\":"+before+",\"shared\":"+shared+",\"removed_count\":"+removed+",\"after\":"+after+",\"repeat_count\":"+repeat+"}");}
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged original image through HEADS then private checkLedgers; actual live objects before serialization, no native outputs\",\"systems\":["+String.join(",",systems)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
