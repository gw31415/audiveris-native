// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import java.awt.Rectangle;import com.fasterxml.jackson.databind.ObjectMapper;
public class DotOwnerEditProbe extends DotPartOwnershipProbe {
 static Object state(Object sig,List<?> stacks,List<Object> dots)throws Exception {
  List<Object> members=new ArrayList<>(),nodes=new ArrayList<>();
  for(Object stack:stacks){Object m=((List<?>)call(stack,"getMeasures")).get(0);List<Object> order=new ArrayList<>();for(Object d:(Collection<?>)call(m,"getAugmentationDots"))if(dots.contains(d))order.add(dots.indexOf(d));members.add(order);}
  Object measure=((List<?>)call(stacks.get(0),"getMeasures")).get(0);
  for(Object d:dots)nodes.add(obj("bounds",rect(call(d,"getBounds")),"grade",call(d,"getGrade"),"contextual_grade",call(d,"getContextualGrade")));
  return obj("ownership",ownership(sig,measure,dots),"measure_order",members,"nodes",nodes);
 }
 static void watch(Object sig,List<?> stacks,List<Object> dots,List<Object> edges,List<Object> trace)throws Exception {
  java.lang.reflect.Field f=Class.forName("org.jgrapht.graph.DefaultListenableGraph").getDeclaredField("graphListeners");f.setAccessible(true);List<Object> listeners=(List<Object>)f.get(sig);int[] depth={0};int found=0;
  for(int i=0;i<listeners.size();i++){Object delegate=listeners.get(i);if(!delegate.getClass().getSimpleName().equals("SigListener"))continue;found++;
   listeners.set(i,java.lang.reflect.Proxy.newProxyInstance(delegate.getClass().getClassLoader(),new Class[]{Class.forName("org.jgrapht.event.GraphListener")},(p,m,args)->{
    if(!m.getName().equals("edgeRemoved")&&!m.getName().equals("vertexRemoved"))return m.invoke(delegate,args);
    Object entity=call(args[0],m.getName().equals("edgeRemoved")?"getEdge":"getVertex");String label=m.getName().equals("edgeRemoved")?"edge"+edges.indexOf(entity):"dot"+dots.indexOf(entity);int level=depth[0]++;
    trace.add(obj("event",m.getName(),"stage","enter","entity",label,"depth",level,"state",state(sig,stacks,dots)));
    try{return m.invoke(delegate,args);}finally{depth[0]--;trace.add(obj("event",m.getName(),"stage","leave","entity",label,"depth",level,"state",state(sig,stacks,dots)));}
   }));
  }if(found!=1)throw new IllegalStateException("expected original SigListener");
 }
 public static void main(String[] args)throws Exception {
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),dd=null,aug=null;
  for(Object e:(Collection<?>)call(sig,"edgeSet"))if(e.getClass().getSimpleName().equals("DoubleDotRelation")){dd=e;break;}
  Object first=call(sig,"getEdgeTarget",dd),second=call(sig,"getEdgeSource",dd);for(Object e:(Collection<?>)call(sig,"edgesOf",first))if(e.getClass().getSimpleName().equals("AugmentationRelation")){aug=e;break;}
  Object head=call(sig,"getEdgeTarget",aug),part=call(head,"getPart");List<?> stacks=(List<?>)call(sys,"getStacks");List<Object> dots=List.of(first,second),steps=new ArrayList<>(),trace=new ArrayList<>();
  Object before=state(sig,stacks,dots);call(first,"setPart",part);steps.add(obj("action","setPart(existing)","state",state(sig,stacks,dots)));
  call(first,"setStaff",(Object)null);steps.add(obj("action","setStaff(null)","state",state(sig,stacks,dots)));
  call(first,"setGrade",.4);steps.add(obj("action","setGrade(0.4)","state",state(sig,stacks,dots)));
  call(first,"setPart",(Object)null);steps.add(obj("action","setPart(null)","state",state(sig,stacks,dots)));
  call(first,"getPart");steps.add(obj("action","getPart(support)","state",state(sig,stacks,dots)));
  Rectangle b=(Rectangle)call(first,"getBounds");int x=(Integer)call(stacks.get(1),"getLeft")+1;call(first,"setBounds",new Rectangle(x-b.width/2,b.y,b.width,b.height));steps.add(obj("action","setBounds(next stack)","state",state(sig,stacks,dots)));
  watch(sig,stacks,dots,List.of(aug,dd),trace);call(first,"remove");steps.add(obj("action","remove moved dot","state",state(sig,stacks,dots)));
  call(second,"remove");steps.add(obj("action","remove second","state",state(sig,stacks,dots)));
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","declared owner setters on actual saved Inter then actual removal callbacks; no automatic image or multi-staff claim","before",before,"steps",steps,"trace",trace));System.exit(0);
 }
}
