// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.Rectangle;
import java.awt.geom.Line2D;
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class FoundationGridWeakProbe extends VoiceLifecycleProbe {
 public static void main(String[] args)throws Exception{
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0);Object staff=((List<?>)call(sys,"getStaves")).get(1),part=call(staff,"getPart");List<?> partStaffs=new ArrayList<>((List<?>)call(part,"getStaves"));List<Object> cases=new ArrayList<>();
  for(String name:List.of("both-below","brace-boundary","connector-boundary","both-strong","brace-frozen-below","connector-frozen-below","both-frozen-below","reverse-registration-both-below")){
   Object sig=make("sig.SIGraph",sys);Object brace=make("sig.inter.BraceInter",.8);call(brace,"setBounds",new Rectangle(100,100,12,20));call(brace,"setStaff",staff);Object connector=make("sig.inter.BarConnectorInter",c("glyph.Shape").getField("THIN_CONNECTOR").get(null),.8,new Line2D.Double(130.,100.,130.,120.),4.);
   double bg=name.equals("both-strong")?.8:name.equals("brace-boundary")?.5:.499,cg=name.equals("both-strong")?.8:name.equals("connector-boundary")?.5:.499;call(brace,"setContextualGrade",bg);call(connector,"setContextualGrade",cg);boolean bf=name.contains("brace-frozen")||name.contains("both-frozen"),cf=name.contains("connector-frozen")||name.contains("both-frozen");if(bf)call(brace,"freeze");if(cf)call(connector,"freeze");
   List<Object> order=name.startsWith("reverse")?List.of(connector,brace):List.of(brace,connector);for(Object n:order)call(sig,"addVertex",n);Object rel=make("sig.relation.NoExclusion");call(rel,"setManual",true);call(sig,"addEdge",brace,connector,rel);
   List<Object> nodes=new ArrayList<>();for(Object n:order){boolean isBrace=n==brace;Map<String,Object> row=obj("label",isBrace?"brace":"connector","kind",n.getClass().getSimpleName(),"shape",call(n,"getShape").toString(),"bounds",rect(call(n,"getBounds")),"grade",call(n,"getGrade"),"contextual_grade",call(n,"getContextualGrade"),"frozen",call(n,"isFrozen"),"manual",call(n,"isManual"));if(!isBrace){Line2D m=(Line2D)call(n,"getMedian");row.put("median",List.of(m.getX1(),m.getY1(),m.getX2(),m.getY2()));row.put("width",call(n,"getWidth"));}nodes.add(row);}
   List<Object> removed=new ArrayList<>();for(Object n:(Collection<?>)call(sig,"deleteWeakInters"))removed.add(n==brace?"brace":"connector");List<Object> live=new ArrayList<>();for(Object n:order)if((Boolean)call(sig,"containsVertex",n))live.add(n==brace?"brace":"connector");boolean partRetained=call(staff,"getPart")==part&&((List<?>)call(part,"getStaves")).equals(partStaffs);List<?> second=new ArrayList<>((Collection<?>)call(sig,"deleteWeakInters"));
   cases.add(obj("name",name,"nodes",nodes,"removed_order",removed,"live",live,"brace_removed",call(brace,"isRemoved"),"connector_removed",call(connector,"isRemoved"),"same_original_part_and_staff_members",partRetained,"relation_registered",call(sig,"containsEdge",rel),"relation_manual_retained",call(rel,"isManual"),"repeat_removed_count",second.size()));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original automatic Brace/BarConnector weak deletion on isolated SIG with actual original Staff/Part objects; contextual boundary, frozen exclusions, registration order, manual relation and automatic Part preservation; manual Brace structural editing excluded","cases",cases));System.exit(0);
 }
}
