// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealFlagRestManualImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealFlagRestManualImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static String body(Object n)throws Exception {return "{\"id\":"+call(n,"getId")+",\"kind\":\""+n.getClass().getSimpleName()+"\",\"bounds\":"+rect(call(n,"getBounds"))+",\"grade\":"+call(n,"getGrade")+",\"contextual_grade\":"+call(n,"getContextualGrade")+",\"manual\":"+call(n,"isManual")+",\"removed\":"+call(n,"isRemoved")+"}";}
 public void process(){try {
 Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("SYMBOLS").get(null),false);Object sheet=call(stub,"getSheet");List<String> systems=new ArrayList<>();
 for(Object sys:(List<?>)call(sheet,"getSystems")){Object sig=call(sys,"getSig");call(sig,"contextualize");List<Object[]> edges=new ArrayList<>();List<String> before=new ArrayList<>();
 for(Object rel:new ArrayList<>((Set<?>)call(sig,"edgeSet")))if(rel.getClass().getSimpleName().equals("Exclusion")){Object a=call(sig,"getEdgeSource",rel),b=call(sig,"getEdgeTarget",rel);if(a.getClass().getSimpleName().equals("FlagInter")&&b.getClass().getSimpleName().equals("RestInter")){call(b,"setManual",true);call(rel,"setManual",true);edges.add(new Object[]{a,b,rel});before.add("{\"flag\":"+body(a)+",\"rest\":"+body(b)+",\"manual_relation\":"+call(rel,"isManual")+",\"same_glyph_object\":"+(call(a,"getGlyph")==call(b,"getGlyph"))+"}");}}
 call(sig,"deleteWeakInters");List<String> after=new ArrayList<>();for(Object[] e:edges)after.add("{\"flag\":"+body(e[0])+",\"rest\":"+body(e[1])+",\"same_relation_still_present\":"+call(sig,"containsEdge",e[2])+"}");systems.add("{\"system\":"+call(sys,"getId")+",\"before_weak_purge\":["+String.join(",",before)+"],\"after_weak_purge\":["+String.join(",",after)+"]}");}
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged image normal SYMBOLS then whole-SIG contextualize/deleteWeakInters; declared manual Rest and manual Exclusion on actual Flag/Rest same-glyph objects, not full reduceLinks, no native inputs\",\"systems\":["+String.join(",",systems)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
