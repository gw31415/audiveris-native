// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;import java.awt.*;import java.awt.geom.Point2D;
public class RealDotStackImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealDotStackImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static Object field(Object o,String name)throws Exception {for(Class<?> k=o.getClass();k!=null;k=k.getSuperclass())try{var f=k.getDeclaredField(name);f.setAccessible(true);return f.get(o);}catch(NoSuchFieldException e){}throw new NoSuchFieldException(name);}
 static String state(Object sys,Object dot)throws Exception {
  Object part=field(dot,"part");java.util.List<String> members=new ArrayList<>();
  java.util.List<?> stacks=(java.util.List<?>)call(sys,"getStacks");
  for(int i=0;i<stacks.size();i++)for(Object m:(java.util.List<?>)call(stacks.get(i),"getMeasures"))if(((Set<?>)call(m,"getAugmentationDots")).contains(dot))members.add("{\"stack_index\":"+(i+1)+",\"part\":"+call(call(m,"getPart"),"getId")+"}");
  return "{\"cached_part\":"+(part==null?"null":call(part,"getId"))+",\"members\":["+String.join(",",members)+"]}";
 }
 static String query(Object sys,Point2D p)throws Exception {
  Object stack=call(sys,"getStackAt",p);return "{\"point\":"+(p==null?"null":"["+p.getX()+","+p.getY()+"]")+",\"stack_index\":"+(stack==null?"null":((java.util.List<?>)call(sys,"getStacks")).indexOf(stack)+1)+"}";
 }
 public void process(){try {
  Object stub=((java.util.List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("CHORDS").get(null),false);Object sheet=call(stub,"getSheet");java.util.List<String> systems=new ArrayList<>();
  Object left=c("util.HorizontalSide").getField("LEFT").get(null),right=c("util.HorizontalSide").getField("RIGHT").get(null);
  for(Object sys:(java.util.List<?>)call(sheet,"getSystems")) {
   java.util.List<String> queries=new ArrayList<>(),controls=new ArrayList<>();queries.add(query(sys,null));queries.add(query(sys,new Point2D.Double(0,-1000)));
   for(Object part:(java.util.List<?>)call(sys,"getParts"))for(Object staff:(java.util.List<?>)call(part,"getStaves"))for(Object m:(java.util.List<?>)call(part,"getMeasures")) {
    int x1=((Number)call(m,"getAbscissa",left,staff)).intValue(),x2=((Number)call(m,"getAbscissa",right,staff)).intValue();
    for(int x:new int[]{x1,x1+1,x2,x2+1}){double y=((Number)call(call(staff,"getMidLine"),"yAt",(double)x)).doubleValue();queries.add(query(sys,new Point2D.Double(x,Math.rint(y))));}
    int x=(x1+x2)/2,y=(int)Math.rint(((Number)call(call(staff,"getMidLine"),"yAt",(double)x)).doubleValue());
    for(boolean noStaff:new boolean[]{false,true}) {
     Object dot=make("sig.inter.AugmentationDotInter",null,Double.valueOf(.8));call(dot,"setBounds",new Rectangle(x-1,y-1,2,2));if(!noStaff)call(dot,"setStaff",staff);
     String plain=state(sys,dot);Object sig=call(sys,"getSig");call(sig,"addVertex",dot);String added=state(sys,dot);Object stack=call(sys,"getStackAt",call(dot,"getCenter"));if(stack!=null)call(stack,"addInter",dot);String populated=state(sys,dot);call(dot,"remove");String removed=state(sys,dot);
     controls.add("{\"point\":["+x+","+y+"],\"staff\":"+(noStaff?"null":call(staff,"getId"))+",\"plain\":"+plain+",\"added\":"+added+",\"populated\":"+populated+",\"removed\":"+removed+"}");
    }
   }
   systems.add("{\"system\":"+call(sys,"getId")+",\"queries\":["+String.join(",",queries)+"],\"controls\":["+String.join(",",controls)+"]}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged image through CHORDS actual Staff/Part/Measure/Stack; boundary queries and declared new dot add/populate/remove controls, no native input\",\"systems\":["+String.join(",",systems)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
