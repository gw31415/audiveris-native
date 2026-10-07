// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class DotSupportReaddProbe extends DotOwnerEditProbe {
 static Object supportState(Object sig,List<?> stacks,List<Object> notes,List<Object> chords,List<Object> dots,List<Object> edges)throws Exception {
  List<Object> incident=new ArrayList<>();if((Boolean)call(sig,"containsVertex",dots.get(0)))for(Object e:(Collection<?>)call(sig,"edgesOf",dots.get(0)))if(edges.contains(e))incident.add(edges.indexOf(e));
  return obj("owners",DotCallbackOrderProbe.state(sig,notes,chords,dots,edges),"ownership",state(sig,stacks,dots),"incident_order",incident);
 }
 static void supportWatch(Object sig,List<?> stacks,List<Object> notes,List<Object> chords,List<Object> dots,List<Object> edges,List<Object> trace)throws Exception {
  java.lang.reflect.Field f=Class.forName("org.jgrapht.graph.DefaultListenableGraph").getDeclaredField("graphListeners");f.setAccessible(true);List<Object> listeners=(List<Object>)f.get(sig);int[] depth={0};int found=0;
  for(int i=0;i<listeners.size();i++){Object delegate=listeners.get(i);if(!delegate.getClass().getSimpleName().equals("SigListener"))continue;found++;
   listeners.set(i,java.lang.reflect.Proxy.newProxyInstance(delegate.getClass().getClassLoader(),new Class[]{Class.forName("org.jgrapht.event.GraphListener")},(p,m,args)->{
    if(!m.getName().equals("edgeRemoved")&&!m.getName().equals("vertexRemoved"))return m.invoke(delegate,args);
    Object entity=call(args[0],m.getName().equals("edgeRemoved")?"getEdge":"getVertex");String label=m.getName().equals("edgeRemoved")?"edge"+edges.indexOf(entity):"dot"+dots.indexOf(entity);int level=depth[0]++;
    trace.add(obj("event",m.getName(),"stage","enter","entity",label,"depth",level,"state",supportState(sig,stacks,notes,chords,dots,edges)));
    try{return m.invoke(delegate,args);}finally{depth[0]--;trace.add(obj("event",m.getName(),"stage","leave","entity",label,"depth",level,"state",supportState(sig,stacks,notes,chords,dots,edges)));}
   }));
  }if(found!=1)throw new IllegalStateException("expected original SigListener");
 }
 public static void main(String[] args)throws Exception {
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),dd=null,aug=null;
  for(Object e:(Collection<?>)call(sig,"edgeSet"))if(e.getClass().getSimpleName().equals("DoubleDotRelation")){dd=e;break;}
  Object first=call(sig,"getEdgeTarget",dd),second=call(sig,"getEdgeSource",dd);for(Object e:(Collection<?>)call(sig,"edgesOf",first))if(e.getClass().getSimpleName().equals("AugmentationRelation")){aug=e;break;}
  Object h0=call(sig,"getEdgeTarget",aug),c0=call(h0,"getChord"),h1=null;for(Object n:(Collection<?>)call(sig,"vertexSet"))if(n.getClass().getSimpleName().equals("HeadInter")&&call(n,"getChord")!=c0&&call(call(n,"getChord"),"getMeasure")==call(c0,"getMeasure")){h1=n;break;}
  if(h1==null)throw new IllegalStateException("distinct actual owner required");Object c1=call(h1,"getChord"),other=edge(sig,first,h1,"AugmentationRelation",.7);List<?> stacks=(List<?>)call(sys,"getStacks");List<Object> notes=List.of(h0,h1),chords=List.of(c0,c1),dots=List.of(first,second),edges=List.of(aug,other,dd),steps=new ArrayList<>(),trace=new ArrayList<>();
  for(Object c:chords)call(c,"countDots");call(h0,"setPart",(Object)null);call(h0,"setStaff",(Object)null);call(first,"setPart",(Object)null);
  Object before=supportState(sig,stacks,notes,chords,dots,edges);Object cold=call(first,"getPart");steps.add(obj("action","getPart before readd","result",cold==null?null:call(cold,"getId"),"state",supportState(sig,stacks,notes,chords,dots,edges)));
  supportWatch(sig,stacks,notes,chords,dots,edges,trace);call(sig,"removeEdge",aug);steps.add(obj("action","remove first support","state",supportState(sig,stacks,notes,chords,dots,edges)));
  call(sig,"addEdge",first,h0,aug);steps.add(obj("action","readd same support","state",supportState(sig,stacks,notes,chords,dots,edges)));
  Object again=call(sig,"addEdge",first,h0,aug);steps.add(obj("action","repeat live support","added",again,"state",supportState(sig,stacks,notes,chords,dots,edges)));
  call(first,"setPart",(Object)null);Object warm=call(first,"getPart");steps.add(obj("action","getPart after readd","result",warm==null?null:call(warm,"getId"),"state",supportState(sig,stacks,notes,chords,dots,edges)));
  call(first,"remove");steps.add(obj("action","remove dot","state",supportState(sig,stacks,notes,chords,dots,edges)));
  List<Object> facts=new ArrayList<>();for(Object h:notes)facts.add(obj("kind","HeadInter","shape",call(h,"getShape").toString(),"bounds",rect(call(h,"getBounds")),"grade",call(h,"getGrade")));
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","one declared shared support on actual saved owners, Part reset and same relation remove/re-add; all callback stages, not new image admission","notes",facts,"owner_bounds",List.of(rect(call(c0,"getBounds")),rect(call(c1,"getBounds"))),"before",before,"steps",steps,"trace",trace));System.exit(0);
 }
}
