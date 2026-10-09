// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealContextProjectionImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealContextProjectionImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static boolean focus(Object n){return Set.of("HeadInter","StemInter","BeamInter","BeamHookInter","SmallBeamInter","FlagInter").contains(n.getClass().getSimpleName());}
 static String contexts(Object sig)throws Exception {List<String> rows=new ArrayList<>();for(Object inter:(Set<?>)call(sig,"vertexSet"))if(focus(inter))rows.add("{\"id\":"+call(inter,"getId")+",\"kind\":\""+inter.getClass().getSimpleName()+"\",\"bounds\":"+rect(call(inter,"getBounds"))+",\"grade\":"+call(inter,"getGrade")+",\"contextual_grade\":"+call(sig,"computeContextualGrade",inter)+"}");return "["+String.join(",",rows)+"]";}
 public void process(){try {
 Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("SYMBOLS").get(null),false);Object sheet=call(stub,"getSheet");List<String> systems=new ArrayList<>();
 for(Object sys:(List<?>)call(sheet,"getSystems")){
 Object sig=call(sys,"getSig");String before=contexts(sig);List<Object[]> saved=new ArrayList<>();List<String> controls=new ArrayList<>();
 for(Object edge:new ArrayList<>((Set<?>)call(sig,"edgeSet")))if(Set.of("Exclusion","NoExclusion").contains(edge.getClass().getSimpleName())){Object a=call(sig,"getEdgeSource",edge),b=call(sig,"getEdgeTarget",edge);if(focus(a)!=focus(b)){saved.add(new Object[]{a,b,edge});controls.add("{\"kind\":\""+edge.getClass().getSimpleName()+"\",\"source_kind\":\""+a.getClass().getSimpleName()+"\",\"source_bounds\":"+rect(call(a,"getBounds"))+",\"target_kind\":\""+b.getClass().getSimpleName()+"\",\"target_bounds\":"+rect(call(b,"getBounds"))+",\"manual\":"+call(edge,"isManual")+"}");call(sig,"removeEdge",edge);}}
 String without=contexts(sig);boolean identities=true;for(Object[] e:saved){call(sig,"addEdge",e[0],e[1],e[2]);identities&=(Boolean)call(sig,"containsEdge",e[2]);identities&=call(sig,"getEdgeSource",e[2])==e[0]&&call(sig,"getEdgeTarget",e[2])==e[1];}String restored=contexts(sig);
 systems.add("{\"system\":"+call(sys,"getId")+",\"external_non_support_edges\":["+String.join(",",controls)+"],\"before\":"+before+",\"without_external_non_support_edges\":"+without+",\"restored\":"+restored+",\"same_relation_objects_restored\":"+identities+"}");}
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"unchanged original image normal SYMBOLS; compute Head/Stem/Beam/Flag contextual grades before and without external non-Support edges and after same-object restoration; no native inputs\",\"systems\":["+String.join(",",systems)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
