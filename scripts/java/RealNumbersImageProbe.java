// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;import java.awt.Point;
public class RealNumbersImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealNumbersImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static String rows(Object sheet,boolean links)throws Exception{
  List<String> systems=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){
   Object sig=call(sys,"getSig");List<String> rows=new ArrayList<>();
   for(String type:List.of("NumberInter","TimeNumberInter","TimePairInter","MeasureCountInter"))for(Object n:(Collection<?>)call(sig,"inters",c("sig.inter."+type))){
    Object staff=call(n,"getStaff"),g=call(n,"getGlyph"),center=call(n,"getCenter");Point centroid=g==null?null:(Point)call(g,"getCentroid");List<String> candidates=new ArrayList<>();
    if(type.equals("NumberInter")&&links)for(Object link:(Collection<?>)call(n,"searchLinks",sys)){var f=link.getClass().getField("partner");Object partner=f.get(link);candidates.add("{\"kind\":\""+partner.getClass().getSimpleName()+"\",\"bounds\":"+rect(call(partner,"getBounds"))+"}");}
    rows.add("{\"kind\":\""+type+"\",\"id\":"+call(n,"getId")+",\"shape\":\""+call(n,"getShape")+"\",\"bounds\":"+rect(call(n,"getBounds"))+",\"staff\":"+(staff==null?"null":call(staff,"getId"))+",\"centroid\":"+(centroid==null?"null":"["+centroid.x+","+centroid.y+"]")+",\"centroid_pitch\":"+(staff==null||centroid==null?"null":call(staff,"pitchPositionOf",centroid))+",\"center_pitch\":"+(staff==null?"null":call(staff,"pitchPositionOf",center))+",\"center_inside_staff\":"+(staff==null?"null":call(staff,"contains",center))+",\"grade\":"+call(n,"getGrade")+",\"manual\":"+call(n,"isManual")+",\"abnormal\":"+call(n,"isAbnormal")+",\"frozen\":"+call(n,"isFrozen")+",\"search_links\":["+String.join(",",candidates)+"]}");
   }
   systems.add("{\"system\":"+call(sys,"getId")+",\"numbers\":["+String.join(",",rows)+"]}");
  }
  return "["+String.join(",",systems)+"]";
 }
 public void process(){try{
  Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("SYMBOLS").get(null),false);Object sheet=call(stub,"getSheet");String before=rows(sheet,true);List<Object> held=new ArrayList<>();for(Object sys:(List<?>)call(sheet,"getSystems"))held.addAll((Collection<?>)call(call(sys,"getSig"),"inters",c("sig.inter.NumberInter")));
  call(stub,"reachStep",c("step.OmrStep").getField("LINKS").get(null),false);List<String> removed=new ArrayList<>();for(Object n:held)removed.add("{\"id\":"+call(n,"getId")+",\"removed\":"+call(n,"isRemoved")+"}");
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged original PNG ordinary SYMBOLS then LINKS; Number searchLinks observer and retained identity only; no native input\",\"SYMBOLS\":"+before+",\"LINKS\":"+rows(sheet,false)+",\"retained_number_after_LINKS\":["+String.join(",",removed)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
