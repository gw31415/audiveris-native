// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;
import java.util.*;
import java.awt.geom.Point2D;
public class RealMeasuresImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealMeasuresImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static String bars(List<?> members)throws Exception{List<String> rows=new ArrayList<>();for(Object n:members)rows.add("{\"bounds\":"+rect(call(n,"getBounds"))+",\"shape\":\""+call(n,"getShape")+"\",\"grade\":"+call(n,"getGrade")+",\"frozen\":"+call(n,"isFrozen")+",\"manual\":"+call(n,"isManual")+"}");return "["+String.join(",",rows)+"]";}
 public void process(){try{
  Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("TEXTS").get(null),false);Object sheet=call(stub,"getSheet");List<String> systems=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){Object builder=make("sheet.rhythm.MeasuresBuilder",sys);List<String> groups=new ArrayList<>(),staffbars=new ArrayList<>(),parts=new ArrayList<>();List<?> actualParts=new ArrayList<>((List<?>)call(sys,"getParts"));
   for(Object staff:(List<?>)call(sys,"getStaves")){List<String> gs=new ArrayList<>();for(Object g:(List<?>)call(builder,"buildGroups",call(staff,"getBarlines"))){Object center=getField(g,"center"),dsk=getField(g,"dsk");gs.add("{\"members\":"+bars((List<?>)g)+",\"center\":["+((Point2D)center).getX()+","+((Point2D)center).getY()+"],\"deskew_x\":"+((Point2D)dsk).getX()+"}");}groups.add("{\"staff\":"+call(staff,"getId")+",\"groups\":["+String.join(",",gs)+"]}");}
   call(builder,"buildMeasures");for(Object staff:(List<?>)call(sys,"getStaves")){List<String> bs=new ArrayList<>();for(Object sb:(List<?>)call(staff,"getStaffBarlines")){String style=call(sb,"getStyle")==null?"null":"\""+call(sb,"getStyle")+"\"";bs.add("{\"bounds\":"+rect(call(sb,"getBounds"))+",\"members\":"+bars((List<?>)call(sb,"getMembers"))+",\"style\":"+style+",\"grade\":"+call(sb,"getGrade")+",\"frozen\":"+call(sb,"isFrozen")+",\"manual\":"+call(sb,"isManual")+",\"back_to_back\":"+call(sb,"isBackToBack")+"}");}staffbars.add("{\"staff\":"+call(staff,"getId")+",\"bars\":["+String.join(",",bs)+"]}");}
   for(Object part:actualParts){if(!((List<?>)call(sys,"getParts")).contains(part))throw new IllegalStateException("Part replaced");List<String> ids=new ArrayList<>();for(Object staff:(List<?>)call(part,"getStaves"))ids.add(call(staff,"getId").toString());parts.add("{\"part\":"+call(part,"getId")+",\"staffs\":["+String.join(",",ids)+"],\"measure_count\":"+((List<?>)call(part,"getMeasures")).size()+"}");}
   systems.add("{\"system\":"+call(sys,"getId")+",\"groups\":["+String.join(",",groups)+"],\"staff_bars\":["+String.join(",",staffbars)+"],\"parts\":["+String.join(",",parts)+"],\"stack_count\":"+((List<?>)call(sys,"getStacks")).size()+"}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"unchanged original normal reachStep TEXTS before MEASURES in memory without intermediate SIG serialization and original MeasuresBuilder; actual Part/Staff/bar groups and StaffBarline members before CHORDS, no native outputs\",\"systems\":["+String.join(",",systems)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
 static Object getField(Object o,String name)throws Exception{var f=o.getClass().getDeclaredField(name);f.setAccessible(true);return f.get(o);}
}
