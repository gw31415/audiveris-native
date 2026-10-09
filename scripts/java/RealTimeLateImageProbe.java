// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealTimeLateImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealTimeLateImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static String body(Object n)throws Exception {Object staff=call(n,"getStaff");return "{\"id\":"+call(n,"getId")+",\"kind\":\""+n.getClass().getSimpleName()+"\",\"bounds\":"+rect(call(n,"getBounds"))+",\"grade\":"+call(n,"getGrade")+",\"manual\":"+call(n,"isManual")+",\"removed\":"+call(n,"isRemoved")+",\"staff\":"+(staff==null?"null":call(staff,"getId"))+"}";}
 public void process(){try {
 Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("CHORDS").get(null),false);Object sheet=call(stub,"getSheet");Map<Object,List<?>> optionals=new LinkedHashMap<>();call(make("sheet.symbol.SymbolsFilter",sheet),"process",optionals);List<String> systems=new ArrayList<>();
 for(Object sys:(List<?>)call(sheet,"getSystems")){
 Object sig=call(sys,"getSig"),factory=make("sheet.symbol.InterFactory",sys);call(make("sheet.symbol.SymbolsBuilder",sys,factory),"buildSymbols",optionals);call(make("sheet.note.ChordsBuilder",sys),"buildRestChords");var f=factory.getClass().getDeclaredField("dotFactory");f.setAccessible(true);call(f.get(factory),"lateDotChecks");call(factory,"handleComplexDynamics");
 List<Object> all=new ArrayList<>((Set<?>)call(sig,"vertexSet"));List<String> before=new ArrayList<>(),times=new ArrayList<>();
 for(Object n:all){before.add(body(n));if(Set.of("TimeWholeInter","TimeCustomInter","TimeNumberInter").contains(n.getClass().getSimpleName())){Object staff=call(n,"getStaff"),center=call(n,"getCenter"),stack=call(sys,"getStackAt",center);times.add("{\"body\":"+body(n)+",\"header_stop\":"+call(staff,"getHeaderStop")+",\"center_x\":"+((java.awt.Point)center).x+",\"stack_id\":"+(stack==null?"null":call(stack,"getIdValue"))+"}");}}
 call(factory,"handleTimes");List<String> after=new ArrayList<>();for(Object n:all)after.add(body(n));systems.add("{\"system\":"+call(sys,"getId")+",\"times\":["+String.join(",",times)+"],\"before\":["+String.join(",",before)+"],\"after\":["+String.join(",",after)+"]}");}
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"unchanged image SymbolsBuilder then RestChords then lateDotChecks then handleComplexDynamics; actual handleTimes before/after, removed original object references retained, no native inputs\",\"systems\":["+String.join(",",systems)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
