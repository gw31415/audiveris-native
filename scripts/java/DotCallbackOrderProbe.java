// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.lang.reflect.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class DotCallbackOrderProbe extends DotCompetitionProbe {
 static Object state(Object sig,List<Object> notes,List<Object> chords,List<Object> dots,List<Object> edges)throws Exception {
  List<Object> ns=new ArrayList<>(),cs=new ArrayList<>(),ds=new ArrayList<>(),es=new ArrayList<>();
  for(Object n:notes)ns.add(call(n,"getDotCount"));for(Object c:chords)cs.add(call(c,"getDotsNumber"));
  for(Object d:dots)ds.add(obj("removed",call(d,"isRemoved"),"registered",call(sig,"containsVertex",d),"abnormal",call(d,"isAbnormal")));
  for(Object e:edges)es.add(((Collection<?>)call(sig,"edgeSet")).contains(e));
  return obj("note_dot_counts",ns,"chord_dot_counts",cs,"dots",ds,"edges_live",es);
 }
 static void watch(Object sig,List<Object> notes,List<Object> chords,List<Object> dots,List<Object> edges,List<Object> trace)throws Exception {
  Field f=Class.forName("org.jgrapht.graph.DefaultListenableGraph").getDeclaredField("graphListeners");f.setAccessible(true);
  List<Object> listeners=(List<Object>)f.get(sig);int found=0;int[] depth={0};
  for(int i=0;i<listeners.size();i++){
   Object original=listeners.get(i);if(!original.getClass().getSimpleName().equals("SigListener"))continue;found++;
   Object proxy=Proxy.newProxyInstance(original.getClass().getClassLoader(),new Class[]{Class.forName("org.jgrapht.event.GraphListener")},(p,m,args)->{
    if(!m.getName().equals("edgeRemoved")&&!m.getName().equals("vertexRemoved"))return m.invoke(original,args);
    Object event=args[0];boolean edge=m.getName().equals("edgeRemoved");Object entity=call(event,edge?"getEdge":"getVertex");
    String label=edge?"edge"+edges.indexOf(entity):"dot"+dots.indexOf(entity);int level=depth[0]++;
    trace.add(obj("event",m.getName(),"stage","enter","entity",label,"depth",level,"state",state(sig,notes,chords,dots,edges)));
    try { return m.invoke(original,args); }
    finally {depth[0]--;trace.add(obj("event",m.getName(),"stage","leave","entity",label,"depth",level,"state",state(sig,notes,chords,dots,edges)));}
   });listeners.set(i,proxy);
  }
  if(found!=1)throw new IllegalStateException("expected one original SigListener, got "+found);
 }
 public static void main(String[] args)throws Exception {
  List<Object> cases=new ArrayList<>();for(int i=0;i<3;i++){
   Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),rest=null;
   for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(n.getClass().getSimpleName().equals("AugmentationDotInter"))call(n,"remove");
   List<Object> chords=new ArrayList<>();for(Object n:(Collection<?>)call(sig,"vertexSet")){if(n.getClass().getSimpleName().equals("HeadChordInter"))chords.add(n);if(n.getClass().getSimpleName().equals("RestInter"))rest=n;}
   chords.sort(Comparator.comparingInt(n->{try{return ((java.awt.Rectangle)call(n,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
   Object a=chords.get(0),old=chords.get(1),h0=((List<?>)call(a,"getNotes")).get(0),h1=((List<?>)call(old,"getNotes")).get(0),rc=call(rest,"getChord");
   call(old,"removeMember",h1);call(a,"addMember",h1);if(call(h1,"getChord")!=a)throw new IllegalStateException("member owner mismatch");
   List<Object> notes=List.of(h0,h1,rest),owners=List.of(a,rc),dots=new ArrayList<>(),edges=new ArrayList<>();
   for(Object h:List.of(h0,h1)){java.awt.Rectangle b=(java.awt.Rectangle)call(h,"getBounds");Object d=dot(sheet,sig,h,b.x+b.width+12,b.y-5);dots.add(d);call(call(a,"getMeasure"),"addInter",d);}
   if(i==1){edges.add(edge(sig,dots.get(0),rest,"AugmentationRelation",.7));edges.add(edge(sig,dots.get(0),h0,"AugmentationRelation",.7));}
   else {edges.add(edge(sig,dots.get(0),h0,"AugmentationRelation",.7));edges.add(edge(sig,dots.get(0),rest,"AugmentationRelation",.7));}
   edges.add(edge(sig,dots.get(1),h1,"AugmentationRelation",.7));if(i==2)call(dots.get(1),"setManual",true);
   for(Object owner:owners)call(owner,"countDots");Object before=state(sig,notes,owners,dots,edges);List<Object> trace=new ArrayList<>();watch(sig,notes,owners,dots,edges,trace);call(dots.get(0),"remove");Object after=state(sig,notes,owners,dots,edges);
   List<Object> noteFacts=new ArrayList<>();for(Object n:notes)noteFacts.add(obj("kind",n.getClass().getSimpleName(),"bounds",rect(call(n,"getBounds")),"shape",call(n,"getShape").toString()));
   cases.add(obj("rest_edge_first",i==1,"remaining_manual",i==2,"notes",noteFacts,"owner_bounds",owners.stream().map(o->{try{return rect(call(o,"getBounds"));}catch(Exception e){throw new RuntimeException(e);}}).toList(),"member_order",((List<?>)call(a,"getNotes")).stream().map(n->notes.indexOf(n)).toList(),"dot_bounds",dots.stream().map(d->{try{return rect(call(d,"getBounds"));}catch(Exception e){throw new RuntimeException(e);}}).toList(),"edge_targets",i==1?List.of(2,0,1):List.of(0,2,1),"before",before,"trace",trace,"after",after));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","declared shared temporary dots and explicit member transfer on actual saved half-rest/HeadChord owners; original SigListener wrapped in-place, unchanged order and delegates; captured event identity before reuse; raw callbacks, not image recognition/export success","cases",cases));System.exit(0);
 }
}
