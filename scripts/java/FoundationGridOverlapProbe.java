// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.Rectangle;
import java.awt.geom.Line2D;
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class FoundationGridOverlapProbe extends VoiceLifecycleProbe {
 static Object glyph(Object g)throws Exception{if(g==null)return null;Object t=call(g,"getRunTable");List<Object> runs=new ArrayList<>();for(int i=0;i<((Number)call(t,"getSize")).intValue();i++){Iterator<?> it=(Iterator<?>)call(t,"iterator",i);while(it.hasNext()){Object r=it.next();runs.add(List.of(i,call(r,"getStart"),call(r,"getLength")));}}return obj("bounds",rect(call(g,"getBounds")),"orientation",call(t,"getOrientation").toString(),"runs",runs);}
 static Object physical(boolean shared)throws Exception{byte[] pixels=new byte[12*20];Arrays.fill(pixels,(byte)255);if(shared){for(int y:new int[]{0,19})for(int x:new int[]{0,11})pixels[y*12+x]=0;pixels[10*12+6]=0;}else for(int[] p:new int[][]{{0,2},{11,17},{2,0},{9,19}})pixels[p[1]*12+p[0]]=0;Object image=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class,byte[].class).newInstance(12,20,pixels);return make("glyph.Glyph",100,100,call(make("run.RunTableFactory",c("run.Orientation").getField("VERTICAL").get(null)),"createTable",image));}
 static Object row(Object n)throws Exception{Map<String,Object> r=obj("kind",n.getClass().getSimpleName(),"shape",call(n,"getShape").toString(),"bounds",rect(call(n,"getBounds")),"core_bounds",rect(call(n,"getCoreBounds")),"grade",call(n,"getGrade"),"glyph",glyph(call(n,"getGlyph")),"implicit",call(n,"isImplicit"));if(r.get("kind").equals("HeadInter")){r.put("pitch",0.);r.put("staff_id",1);}if(r.get("kind").equals("BarConnectorInter")){Line2D m=(Line2D)call(n,"getMedian");r.put("median",List.of(m.getX1(),m.getY1(),m.getX2(),m.getY2()));r.put("width",call(n,"getWidth"));}return r;}
 public static void main(String[] args)throws Exception{
  List<Object> actual=new ArrayList<>(),cases=new ArrayList<>();Object originalBook=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));Object originalSheet=call(((List<?>)call(originalBook,"getStubs")).get(0),"getSheet");
  for(Object sys:(List<?>)call(originalSheet,"getSystems"))for(Object n:(Collection<?>)call(call(sys,"getSig"),"vertexSet"))if(List.of("BraceInter","BarConnectorInter").contains(n.getClass().getSimpleName())){Map<String,Object> r=(Map<String,Object>)row(n);r.put("system",call(sys,"getId"));actual.add(r);}
  for(String mode:List.of("connector-glyph-contact","connector-glyph-gap","connector-bounds-contact","connector-area-contact","connector-area-boundary","brace-shared-glyph","brace-gap-glyph","brace-no-glyph","NoExclusion","BarConnectionRelation","BarGroupRelation")){
   Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
   boolean brace=mode.startsWith("brace")||!mode.startsWith("connector");Object a;
   if(brace){a=make("sig.inter.BraceInter",.8);call(a,"setBounds",new Rectangle(100,100,12,20));if(!mode.equals("brace-no-glyph"))call(a,"setGlyph",physical(true));}
   else a=make("sig.inter.BarConnectorInter",c("glyph.Shape").getField("THIN_CONNECTOR").get(null),.8,new Line2D.Double(106.,100.,106.,120.),4.);
   Object b;if(mode.startsWith("connector-area"))b=make("sig.inter.BarConnectorInter",c("glyph.Shape").getField("THIN_CONNECTOR").get(null),.8,new Line2D.Double(mode.endsWith("boundary")?110.:109.,100.,mode.endsWith("boundary")?110.:109.,120.),4.);
   else {b=make("sig.inter.HeadInter",new Rectangle(100,100,12,20),c("glyph.Shape").getField("NOTEHEAD_BLACK").get(null),.8,call(sys,"getFirstStaff"),0.);if(!mode.equals("brace-no-glyph")&&!mode.equals("connector-bounds-contact"))call(b,"setGlyph",physical(!mode.contains("gap")));}
   call(sig,"addVertex",a);call(sig,"addVertex",b);Object support=null;
   if(List.of("NoExclusion","BarConnectionRelation","BarGroupRelation").contains(mode)){support=mode.equals("BarGroupRelation")?make("sig.relation.BarGroupRelation",0.):make("sig.relation."+mode);call(support,"setManual",true);call(sig,"addEdge",a,b,support);}
   Object reducer=make("sig.SigReducer",sys,true);List<Object> nodes=List.of(row(a),row(b));List<Object> directional=List.of(call(a,"overlaps",b),call(b,"overlaps",a));boolean noSupport=(Boolean)call(sig,"noSupport",a,b);List<Object> sorted=new ArrayList<>(List.of(a,b));sorted.sort(Comparator.comparingInt(n->((Rectangle)callUnchecked(n,"getBounds")).x));detect(reducer,sorted);
   List<Object> exclusions=new ArrayList<>();Object first=null;for(Object e:(Collection<?>)call(sig,"getAllEdges",a,b))if(e.getClass().getSimpleName().equals("Exclusion")){first=e;exclusions.add(obj("source","a","target","b","cause",field(e,"sig.relation.Exclusion","cause").toString()));}
   detect(reducer,sorted);boolean retained=first==null||((Collection<?>)call(sig,"getAllEdges",a,b)).contains(first);cases.add(obj("name",mode,"nodes",nodes,"directional_overlaps",directional,"no_support",noSupport,"support_kind",support==null?null:mode,"support_manual",support==null?null:call(support,"isManual"),"exclusions",exclusions,"repeat_relation_identity_retained",retained));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual real-image GRID Brace/BarConnector geometry plus declared directional glyph/area/bounds and typed Support versus non-Support controls; not full reduction or quality acceptance","actual",actual,"cases",cases));System.exit(0);
 }
 static void detect(Object reducer,List<Object> sorted)throws Exception{var method=c("sig.SigReducer").getDeclaredMethod("detectOverlaps",List.class);method.setAccessible(true);method.invoke(reducer,sorted);}
 static Object callUnchecked(Object o,String name){try{return call(o,name);}catch(Exception e){throw new RuntimeException(e);}}
}
