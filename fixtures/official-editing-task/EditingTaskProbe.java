// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import java.awt.Rectangle;import java.lang.reflect.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class EditingTaskProbe extends DotPartOwnershipProbe {
 static Object state(Object sig,Object measure,List<Object> notes,List<Object> chords,List<Object> dots,List<Object> edges,Object originalGlyph)throws Exception {
  List<Object> fields=new ArrayList<>();for(Object d:dots)fields.add(obj("bounds",rect(call(d,"getBounds")),"grade",call(d,"getGrade"),"contextual_grade",call(d,"getContextualGrade"),"has_glyph",call(d,"getGlyph")!=null,"attachments",((Map<?,?>)call(d,"getAttachments")).size()));
  return obj("owners",DotCallbackOrderProbe.state(sig,notes,chords,dots,edges),"ownership",ownership(sig,measure,dots),"dot_fields",fields,"original_glyph_registered",((Collection<?>)call(call(call(call(sig,"getSystem"),"getSheet"),"getGlyphIndex"),"getEntities")).contains(originalGlyph));
 }
 static void watch(Object sig,Object measure,List<Object> notes,List<Object> chords,List<Object> dots,List<Object> edges,Object glyph,List<Object> trace)throws Exception {
  Field f=Class.forName("org.jgrapht.graph.DefaultListenableGraph").getDeclaredField("graphListeners");f.setAccessible(true);List<Object> listeners=(List<Object>)f.get(sig);int[] depth={0};
  for(int i=0;i<listeners.size();i++){Object delegate=listeners.get(i);if(!delegate.getClass().getSimpleName().equals("SigListener"))continue;
   listeners.set(i,Proxy.newProxyInstance(delegate.getClass().getClassLoader(),new Class[]{Class.forName("org.jgrapht.event.GraphListener")},(p,m,args)->{
    if(!List.of("edgeAdded","edgeRemoved","vertexRemoved").contains(m.getName()))return m.invoke(delegate,args);
    boolean edge=m.getName().startsWith("edge");Object entity=call(args[0],edge?"getEdge":"getVertex");String label=edge?"edge"+edges.indexOf(entity):"dot"+dots.indexOf(entity);int level=depth[0]++;
    trace.add(obj("event",m.getName(),"stage","enter","entity",label,"depth",level,"state",state(sig,measure,notes,chords,dots,edges,glyph)));
    try{return m.invoke(delegate,args);}finally{depth[0]--;trace.add(obj("event",m.getName(),"stage","leave","entity",label,"depth",level,"state",state(sig,measure,notes,chords,dots,edges,glyph)));}
   }));
  }
 }
 public static void main(String[] args)throws Exception {
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),dd=null,aug=null;
  for(Object e:(Collection<?>)call(sig,"edgeSet"))if(e.getClass().getSimpleName().equals("DoubleDotRelation")){dd=e;break;}
  Object first=call(sig,"getEdgeTarget",dd),second=call(sig,"getEdgeSource",dd);for(Object e:(Collection<?>)call(sig,"edgesOf",first))if(e.getClass().getSimpleName().equals("AugmentationRelation")){aug=e;break;}
  Object h0=call(sig,"getEdgeTarget",aug),c0=call(h0,"getChord"),measure=call(c0,"getMeasure"),h1=null;for(Object n:(Collection<?>)call(sig,"vertexSet"))if(n.getClass().getSimpleName().equals("HeadInter")&&call(n,"getChord")!=c0&&call(call(n,"getChord"),"getMeasure")==measure){h1=n;break;}
  Object c1=call(h1,"getChord"),glyph=call(first,"getGlyph"),fresh=make("sig.relation.AugmentationRelation"),duplicate=make("sig.relation.AugmentationRelation");call(fresh,"setGrade",.7);call(duplicate,"setGrade",.7);
  List<Object> notes=List.of(h0,h1),chords=List.of(c0,c1),dots=List.of(first,second),edges=List.of(aug,fresh,dd),trace=new ArrayList<>(),steps=new ArrayList<>();
  call(first,"addAttachment","lookup",new Rectangle(1,2,3,4));Class<?> unsafeClass=Class.forName("sun.misc.Unsafe");Field uf=unsafeClass.getDeclaredField("theUnsafe");uf.setAccessible(true);Object unsafe=uf.get(null);
  Object editor=unsafeClass.getMethod("allocateInstance",Class.class).invoke(unsafe,c("sig.ui.DefaultEditor"));
  for(Object[] setting:List.of(new Object[]{c("sheet.ui.ObjectEditor"),"object",first},new Object[]{c("sheet.ui.ObjectEditor"),"system",sys},new Object[]{c("sig.ui.DefaultEditor"),"originalBounds",call(first,"getBounds")},new Object[]{c("sig.ui.DefaultEditor"),"latestBounds",call(first,"getBounds")})){Field f=((Class<?>)setting[0]).getDeclaredField((String)setting[1]);f.setAccessible(true);f.set(editor,setting[2]);}
  Field b=c("sig.ui.DefaultEditor").getDeclaredField("latestBounds");b.setAccessible(true);((Rectangle)b.get(editor)).translate(3,0);
  Object task=make("sig.ui.EditingTask",editor,List.of(make("sig.relation.Link",h0,duplicate,true),make("sig.relation.Link",h1,fresh,true)),List.of(make("sig.relation.Link",h0,aug,true)));
  Object index=call(sheet,"getInterIndex"),service=make("sig.ui.InterService",index,call(sheet,"getLocationService"));call(index,"setEntityService",service);
  Class<?> event=Class.forName("org.audiveris.omr.ui.selection.EntityListEvent"),subscriber=Class.forName("org.bushe.swing.event.EventSubscriber");Object sub=Proxy.newProxyInstance(subscriber.getClassLoader(),new Class[]{subscriber},(p,m,a)->{if(m.getName().equals("hashCode"))return System.identityHashCode(p);if(m.getName().equals("equals"))return p==a[0];if(m.getName().equals("toString"))return "EditingTaskObserver";if(m.getName().equals("onEvent"))trace.add(obj("event","publish","stage","delivered","entity",call(a[0],"getEntity")==first?"dot0":"other","state",state(sig,measure,notes,chords,dots,edges,glyph)));return null;});call(service,"subscribeStrongly",event,sub);
  Object before=state(sig,measure,notes,chords,dots,edges,glyph);watch(sig,measure,notes,chords,dots,edges,glyph,trace);
  for(String op:List.of("do","undo","redo")){call(task,op.equals("undo")?"performUndo":"performDo");javax.swing.SwingUtilities.invokeAndWait(()->{});steps.add(obj("operation",op,"retained_links",((Collection<?>)call(task,"getLinks")).size(),"state",state(sig,measure,notes,chords,dots,edges,glyph)));}
  List<Object> facts=new ArrayList<>();for(Object h:notes)facts.add(obj("kind","HeadInter","shape",call(h,"getShape").toString(),"bounds",rect(call(h,"getBounds")),"grade",call(h,"getGrade")));
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","actual saved owners, original DefaultEditor model fields installed headlessly (GUI zoom/tracker constructor excluded), real EditingTask do-undo-redo with duplicate candidate and original Link callbacks; real InterService publication delivery, no rendered GUI or automatic image success/controller impact","notes",facts,"owner_bounds",List.of(rect(call(c0,"getBounds")),rect(call(c1,"getBounds"))),"measure",obj("left",call(call(measure,"getStack"),"getLeft"),"right",call(call(measure,"getStack"),"getRight")),"before",before,"steps",steps,"trace",trace));System.exit(0);
 }
}
