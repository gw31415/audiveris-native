// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealDirectionsImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealDirectionsImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 private static String stage(Object sheet,String name)throws Exception{
  List<String> systems=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){
   Object sig=call(sys,"getSig");List<String> nodes=new ArrayList<>();
   for(String type:List.of("DynamicsInter","PedalInter"))for(Object n:(Collection<?>)call(sig,"inters",c("sig.inter."+type))){
    List<String> relations=new ArrayList<>();
    for(Object e:(Set<?>)call(sig,"edgesOf",n)){
     Object ch=call(sig,"getOppositeInter",n,e);
     String extra="";if(e.getClass().getSimpleName().startsWith("Chord")) extra=",\"source_ratio\":"+call(e,"getSourceRatio")+",\"target_ratio\":"+call(e,"getTargetRatio");
     relations.add("{\"kind\":\""+e.getClass().getSimpleName()+"\",\"manual\":"+call(e,"isManual")+",\"opposite_kind\":\""+ch.getClass().getSimpleName()+"\",\"bounds\":"+rect(call(ch,"getBounds"))+extra+"}");
    }
    Object cg=call(n,"getContextualGrade"),staff=call(n,"getStaff");
    nodes.add("{\"kind\":\""+type+"\",\"shape\":\""+call(n,"getShape")+"\",\"bounds\":"+rect(call(n,"getBounds"))+",\"grade\":"+call(n,"getGrade")+",\"manual\":"+call(n,"isManual")+",\"abnormal\":"+call(n,"isAbnormal")+",\"staff\":"+(staff==null?"null":call(staff,"getId"))+",\"contextual_grade\":"+(cg==null?"null":cg)+",\"relations\":["+String.join(",",relations)+"]}");
   }
   systems.add("{\"system\":"+call(sys,"getId")+",\"directions\":["+String.join(",",nodes)+"]}");
  }
  return "{\"stage\":\""+name+"\",\"systems\":["+String.join(",",systems)+"]}";
 }
 public void process(){try{
  Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("SYMBOLS").get(null),false);Object sheet=call(stub,"getSheet");String symbols=stage(sheet,"SYMBOLS");call(stub,"reachStep",c("step.OmrStep").getField("LINKS").get(null),false);
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged original image actual dynamics/pedal SYMBOLS and ordinary LINKS; no native input\",\"stages\":["+symbols+","+stage(sheet,"LINKS")+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
