// SPDX-License-Identifier: AGPL-3.0-or-later
// Original SIG/InterIndex lifecycle observations, not a native graph parity claim.
import java.util.*;
import java.util.concurrent.atomic.AtomicInteger;
import java.nio.file.*;
import java.lang.reflect.*;
import java.awt.geom.*;
import com.fasterxml.jackson.databind.*;
import ij.process.ByteProcessor;
public class BeamExtendToBeamControlProbe extends GridProbe {
 private final Object sourceBook;
 public BeamExtendToBeamControlProbe(org.audiveris.omr.sheet.Book b,SortedSet<Integer> ids){super(b,ids);sourceBook=b;}
 static Class<?> cls(String n)throws Exception{return BeamImpactsControlProbe.cls(n);}
 static Object make(String n,Object...args)throws Exception{return BeamImpactsControlProbe.make(n,args);}
 static Object call(Object o,String n,Object...args)throws Exception{return BeamImpactsControlProbe.call(o,n,args);}
 static void set(Object o,String n,Object v)throws Exception{BeamImpactsControlProbe.set(o,n,v);}
 static void field(Object target,String name,Object value)throws Exception {
  for(Class<?> c=target.getClass();c!=null;c=c.getSuperclass())try{Field f=c.getDeclaredField(name);f.setAccessible(true);f.set(target,value);return;}catch(NoSuchFieldException e){}
  throw new NoSuchFieldException(name);
 }
 static Object beam(double x1,double x2)throws Exception{return make("sig.inter.BeamInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),new Line2D.Double(x1,40,x2,40),8.);}
 static Map<String,Object> snapshot(Object sig,Object index,List<Object> raw)throws Exception {
  List<Object> live=new ArrayList<>(),ids=new ArrayList<>(),removed=new ArrayList<>(),indexed=new ArrayList<>(),glyphIds=new ArrayList<>(),groups=new ArrayList<>();
  for(Object v:(Collection<?>)call(sig,"vertexSet"))if(raw.contains(v))live.add(raw.indexOf(v)+1);
  for(Object v:raw){int id=(Integer)call(v,"getId");ids.add(id);removed.add(call(v,"isRemoved"));indexed.add(call(index,"getEntity",id)!=null);Object g=call(v,"getGlyph");glyphIds.add(call(g,"getId"));List<String> names=new ArrayList<>();for(Object group:(Collection<?>)call(g,"getGroups"))names.add(group.toString());Collections.sort(names);groups.add(names);}
  int edges=0;for(Object e:(Collection<?>)call(sig,"edgeSet"))if(raw.contains(call(sig,"getEdgeSource",e))&&raw.contains(call(sig,"getEdgeTarget",e)))edges++;
  return Map.of("active_ids",live,"original_ids",ids,"removed",removed,"index_present",indexed,"glyph_ids",glyphIds,"glyph_groups",groups,"edge_count",edges);
 }
 @SuppressWarnings({"unchecked","rawtypes"})
 public void process(){try{
  Field uf=sun.misc.Unsafe.class.getDeclaredField("theUnsafe");uf.setAccessible(true);sun.misc.Unsafe unsafe=(sun.misc.Unsafe)uf.get(null);
  Object sheet=call(((List<?>)call(sourceBook,"getStubs")).get(0),"getSheet");
  call(cls("sheet.grid.GridStep").getConstructor().newInstance(),"doit",sheet);
  Object system=((List<?>)call(sheet,"getSystems")).get(0),index=call(sheet,"getInterIndex"),sig=call(system,"getSig");
  ObjectMapper mapper=new ObjectMapper(); JsonNode inputs=mapper.readTree(Files.readString(Path.of(System.getProperty("native.probe.inputs"))));
  List<Object> results=new ArrayList<>();
  for(JsonNode c:inputs){
   int il=c.get("interline").asInt(),w=c.get("width").asInt(),h=c.get("height").asInt();
   Object scale=make("sheet.Scale",make("sheet.Scale$InterlineScale",il,il,il),make("sheet.Scale$LineScale",2,2,2),null,null,null);
   Object builder=unsafe.allocateInstance(cls("sheet.beam.BeamsBuilder"));set(builder,"stdParams",make("sheet.beam.BeamsBuilder$ItemParameters",scale,8.,false));Object params=make("sheet.beam.BeamsBuilder$Parameters",scale);set(builder,"params",params);set(builder,"sheet",sheet);set(builder,"sig",sig);set(builder,"sheetBox",new java.awt.Rectangle(0,0,w,h));
   ByteProcessor pixels=new ByteProcessor(w,h);Arrays.fill((byte[])pixels.getPixels(),(byte)255);for(JsonNode p:c.get("black_pixels"))pixels.set(p.get(0).asInt(),p.get(1).asInt(),0);set(builder,"pixelFilter",pixels);
   List<Object> raw=new ArrayList<>();
   for(String key:new String[]{"first","second"}){JsonNode b=c.get(key),m=b.get("median");raw.add(make("sig.inter.BeamInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,b.get("distance_impact").asDouble()),new Line2D.Double(m.get(0).asDouble(),m.get(1).asDouble(),m.get(2).asDouble(),m.get(3).asDouble()),b.get("height").asDouble()));}
   Object a=raw.get(0),b=raw.get(1);Line2D am=(Line2D)call(a,"getMedian"),bm=(Line2D)call(b,"getMedian");double gap=am.getX1()<bm.getX1()?bm.getX1()-am.getX2():am.getX1()-bm.getX2();
   int min=(Integer)BeamImpactsControlProbe.get(params,"minBeamsGapX");Map<String,Object> row=new LinkedHashMap<>();row.put("input",c);row.put("gap",gap);row.put("minimum_gap",min);row.put("counts",null);boolean gate=true;
   if(gap>=min){Method middle=builder.getClass().getDeclaredMethod("middleArea",cls("sig.inter.AbstractBeamInter"),cls("sig.inter.AbstractBeamInter"));middle.setAccessible(true);Area area=(Area)middle.invoke(builder,a,b);Object black=make("util.WrappedInteger",0);int total=(Integer)call(make("image.AreaMask",area),"fore",black,pixels),count=(Integer)black.getClass().getField("value").get(black);row.put("counts",new int[]{total,count});gate=(double)count/total>=.6;}
   row.put("gate",gate);
   if(c.path("gate_only").asBoolean()){row.put("accepted",null);results.add(row);continue;}
   byte[] supplied=((byte[])pixels.getPixels()).clone();Arrays.fill((byte[])pixels.getPixels(),(byte)0);
   Method register=builder.getClass().getDeclaredMethod("registerBeam",cls("sig.inter.AbstractBeamInter"));register.setAccessible(true);register.invoke(builder,a);register.invoke(builder,b);set(builder,"rawSystemBeams",raw);System.arraycopy(supplied,0,(byte[])pixels.getPixels(),0,supplied.length);
   Object cause=Enum.valueOf((Class)cls("sig.relation.Exclusion$ExclusionCause"),"OVERLAP");call(sig,"addEdge",a,b,make("sig.relation.Exclusion",cause));
   row.put("before",snapshot(sig,index,raw));
   Method extend=builder.getClass().getDeclaredMethod("extendToBeam",cls("sig.inter.AbstractBeamInter"),cls("sig.inter.AbstractBeamInter"));extend.setAccessible(true);boolean accepted=(Boolean)extend.invoke(builder,a,b);row.put("accepted",accepted);row.put("after",snapshot(sig,index,raw));
   if(accepted){Object merged=raw.get(2);Line2D m=(Line2D)call(merged,"getMedian");row.put("merged",Map.of("median",new double[]{m.getX1(),m.getY1(),m.getX2(),m.getY2()},"height",call(merged,"getHeight"),"grade",call(merged,"getGrade")));}
   for(Object v:raw)if(!(Boolean)call(v,"isRemoved"))call(v,"remove");results.add(row);
  }
  mapper.writeValue(Path.of(System.getProperty("native.probe.output")).toFile(),Map.of("original_method","BeamsBuilder.extendToBeam with real Book/Sheet/SIG", "native_outputs_used",false,"cases",results));System.out.println("original extendToBeam cases="+results.size());
 }catch(Exception e){throw new RuntimeException(e);}
 }
}
