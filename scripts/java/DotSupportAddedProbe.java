// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import java.lang.reflect.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class DotSupportAddedProbe extends DotPartOwnershipProbe {
 static Object addedState(Object sig,List<Object> notes,List<Object> chords,List<Object> dots,List<Object> edges)throws Exception {
  List<Object> fields=new ArrayList<>();for(Object d:dots)fields.add(obj("bounds",rect(call(d,"getBounds")),"grade",call(d,"getGrade"),"contextual_grade",call(d,"getContextualGrade"),"manual",call(d,"isManual")));
  return obj("owners",state(sig,notes,chords,dots,edges),"ownership",ownership(sig,call(chords.get(0),"getMeasure"),dots),"dot_fields",fields);
 }
 static void watchAdded(Object sig,List<Object> notes,List<Object> chords,List<Object> dots,List<Object> edges,List<Object> trace)throws Exception {
  Field f=Class.forName("org.jgrapht.graph.DefaultListenableGraph").getDeclaredField("graphListeners");f.setAccessible(true);List<Object> listeners=(List<Object>)f.get(sig);int[] depth={0};int found=0;
  for(int i=0;i<listeners.size();i++){Object delegate=listeners.get(i);if(!delegate.getClass().getSimpleName().equals("SigListener"))continue;found++;
   listeners.set(i,Proxy.newProxyInstance(delegate.getClass().getClassLoader(),new Class[]{Class.forName("org.jgrapht.event.GraphListener")},(p,m,args)->{
    if(!List.of("edgeAdded","edgeRemoved","vertexRemoved").contains(m.getName()))return m.invoke(delegate,args);
    boolean edge=m.getName().startsWith("edge");Object entity=call(args[0],edge?"getEdge":"getVertex");String label=edge?"edge"+edges.indexOf(entity):"dot"+dots.indexOf(entity);int level=depth[0]++;
    trace.add(obj("event",m.getName(),"stage","enter","entity",label,"depth",level,"state",addedState(sig,notes,chords,dots,edges)));
    try{return m.invoke(delegate,args);}finally{depth[0]--;trace.add(obj("event",m.getName(),"stage","leave","entity",label,"depth",level,"state",addedState(sig,notes,chords,dots,edges)));}
   }));
  }if(found!=1)throw new IllegalStateException("expected original SigListener");
 }
 public static void main(String[] args)throws Exception {
  List<Object> cases=new ArrayList<>();for(String mode:List.of("automatic","manual_dot","manual_relation")){
   Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
   for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(n.getClass().getSimpleName().equals("AugmentationDotInter"))call(n,"remove");
   List<Object> cs=new ArrayList<>();for(Object n:(Collection<?>)call(sig,"vertexSet"))if(n.getClass().getSimpleName().equals("HeadChordInter"))cs.add(n);cs.sort(Comparator.comparingInt(n->{try{return ((java.awt.Rectangle)call(n,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
   Object chord=cs.get(0),old=cs.get(1),h0=((List<?>)call(chord,"getNotes")).get(0),h1=((List<?>)call(old,"getNotes")).get(0);if(mode.equals("manual_relation")){call(old,"removeMember",h1);call(chord,"addMember",h1);}call(chord,"countDots");
   java.awt.Rectangle b=(java.awt.Rectangle)call(h0,"getBounds");Object dot=dot(sheet,sig,h0,b.x+b.width+12,b.y-5);call(dot,"setManual",mode.equals("manual_dot"));Object relation=make("sig.relation.AugmentationRelation");call(relation,"setGrade",.7);call(relation,"setManual",mode.equals("manual_relation"));
   List<Object> notes=mode.equals("manual_relation")?List.of(h0,h1):List.of(h0),chords=List.of(chord),dots=List.of(dot),edges=List.of(relation),trace=new ArrayList<>();Object before=addedState(sig,notes,chords,dots,edges);watchAdded(sig,notes,chords,dots,edges,trace);call(sig,"addEdge",dot,h0,relation);Object after=addedState(sig,notes,chords,dots,edges);
   List<Object> ns=new ArrayList<>();for(Object n:notes)ns.add(obj("kind","HeadInter","shape",call(n,"getShape").toString(),"bounds",rect(call(n,"getBounds")),"grade",call(n,"getGrade")));
   cases.add(obj("mode",mode,"notes",ns,"owner_bounds",rect(call(chord,"getBounds")),"dot_bounds",rect(call(dot,"getBounds")),"measure",obj("left",call(call(call(chord,"getMeasure"),"getStack"),"getLeft"),"right",call(call(call(chord,"getMeasure"),"getStack"),"getRight"),"part_id",call(call(chord,"getPart"),"getId"),"staff_id",call(call(h0,"getStaff"),"getId")),"before",before,"after",after,"trace",trace));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","declared actual two-member HeadChord owner on saved half-rest page; automatic/manual dot/manual relation added callbacks and nested deletion, not image success","cases",cases));System.exit(0);
 }
}
