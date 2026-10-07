// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.lang.reflect.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class DotPartOwnershipProbe extends DotCallbackOrderProbe {
 static Object cached(Object d)throws Exception {Field f=c("sig.inter.AbstractInter").getDeclaredField("part");f.setAccessible(true);Object p=f.get(d);return p==null?null:call(p,"getId");}
 static Object ownership(Object sig,Object measure,List<Object> dots)throws Exception {
  List<Object> rows=new ArrayList<>();for(Object d:dots){Object staff=call(d,"getStaff");rows.add(obj("cached_part_id",cached(d),"staff_id",staff==null?null:call(staff,"getId"),"measure_member",((Collection<?>)call(measure,"getAugmentationDots")).contains(d),"registered",call(sig,"containsVertex",d),"removed",call(d,"isRemoved")));}return rows;
 }
 static void ownerWatch(Object sig,Object measure,List<Object> dots,List<Object> edges,List<Object> trace)throws Exception {
  Field f=Class.forName("org.jgrapht.graph.DefaultListenableGraph").getDeclaredField("graphListeners");f.setAccessible(true);List<Object> listeners=(List<Object>)f.get(sig);int[] depth={0};int found=0;
  for(int i=0;i<listeners.size();i++){Object delegate=listeners.get(i);if(!delegate.getClass().getSimpleName().equals("SigListener"))continue;found++;
   listeners.set(i,Proxy.newProxyInstance(delegate.getClass().getClassLoader(),new Class[]{Class.forName("org.jgrapht.event.GraphListener")},(p,m,args)->{
    if(!m.getName().equals("edgeRemoved")&&!m.getName().equals("vertexRemoved"))return m.invoke(delegate,args);
    Object event=args[0],entity=call(event,m.getName().equals("edgeRemoved")?"getEdge":"getVertex");String label=m.getName().equals("edgeRemoved")?"edge"+edges.indexOf(entity):"dot"+dots.indexOf(entity);int level=depth[0]++;
    trace.add(obj("event",m.getName(),"stage","enter","entity",label,"depth",level,"ownership",ownership(sig,measure,dots)));
    try{return m.invoke(delegate,args);}finally{depth[0]--;trace.add(obj("event",m.getName(),"stage","leave","entity",label,"depth",level,"ownership",ownership(sig,measure,dots)));}
   }));
  }if(found!=1)throw new IllegalStateException("expected original SigListener");
 }
 public static void main(String[] args)throws Exception {
  List<Object> cases=new ArrayList<>();for(String mode:List.of("saved_cold","saved_warm","factory_order_then_FRAT")){
   Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),dd=null,aug=null;
   for(Object e:(Collection<?>)call(sig,"edgeSet"))if(e.getClass().getSimpleName().equals("DoubleDotRelation")){dd=e;break;}Object first=call(sig,"getEdgeTarget",dd),second=call(sig,"getEdgeSource",dd);
   for(Object e:(Collection<?>)call(sig,"edgesOf",first))if(e.getClass().getSimpleName().equals("AugmentationRelation")){aug=e;break;}Object head=call(sig,"getEdgeTarget",aug),chord=call(head,"getChord"),measure=call(chord,"getMeasure"),stack=call(measure,"getStack");
   List<Object> stages=new ArrayList<>();List<Object> dots=new ArrayList<>(List.of(first,second));
   if(mode.equals("factory_order_then_FRAT")){
    Object g0=call(first,"getGlyph"),g1=call(second,"getGlyph"),grade0=call(first,"getGrade"),grade1=call(second,"getGrade"),aGrade=call(aug,"getGrade"),dGrade=call(dd,"getGrade");
    call(first,"remove");call(second,"remove");call(measure,"removeInter",first);call(measure,"removeInter",second);
    first=make("sig.inter.AugmentationDotInter",g0,grade0);second=make("sig.inter.AugmentationDotInter",g1,grade1);dots=new ArrayList<>(List.of(first,second));
    call(sig,"addVertex",first);stages.add(obj("action","SIG.addVertex(first)","ownership",ownership(sig,measure,dots)));
    call(first,"setStaff",call(head,"getStaff"));stages.add(obj("action","first.setStaff","ownership",ownership(sig,measure,dots)));
    aug=edge(sig,first,head,"AugmentationRelation",((Number)aGrade).doubleValue());stages.add(obj("action","first support applied","ownership",ownership(sig,measure,dots)));
    call(sig,"addVertex",second);stages.add(obj("action","SIG.addVertex(second)","ownership",ownership(sig,measure,dots)));
    dd=edge(sig,second,first,"DoubleDotRelation",((Number)dGrade).doubleValue());stages.add(obj("action","second support applied","ownership",ownership(sig,measure,dots)));
    invoke(make("sheet.rhythm.PageRhythm",call(sys,"getPage")),"populateFRATs");stages.add(obj("action","PageRhythm.populateFRATs","ownership",ownership(sig,measure,dots)));
   }else if(mode.equals("saved_warm")){call(first,"getPart");call(second,"getPart");}
   call(chord,"countDots");List<Object> trace=new ArrayList<>();Object before=ownership(sig,measure,dots);ownerWatch(sig,measure,dots,List.of(aug,dd),trace);call(first,"remove");Object afterFirst=ownership(sig,measure,dots);call(second,"remove");Object after=ownership(sig,measure,dots);
   cases.add(obj("mode",mode,"stages",stages,"before",before,"trace",trace,"after_first",afterFirst,"after",after,"head",obj("kind","HeadInter","shape",call(head,"getShape").toString(),"bounds",rect(call(head,"getBounds")),"grade",call(head,"getGrade"),"staff_id",call(call(head,"getStaff"),"getId")),"chord_bounds",rect(call(chord,"getBounds")),"dot_bounds",List.of(rect(call(first,"getBounds")),rect(call(second,"getBounds"))),"measure",obj("part_id",call(call(measure,"getPart"),"getId"),"staff_id",call(call(head,"getStaff"),"getId"),"left",call(stack,"getLeft"),"right",call(stack,"getRight"))));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","actual saved chain cold/warm mutation and explicit factory-order reconstruction from same original glyphs then actual populateFRATs; not new image recognition/export","cases",cases));System.exit(0);
 }
}
