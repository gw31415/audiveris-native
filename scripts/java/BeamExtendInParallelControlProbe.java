// SPDX-License-Identifier: AGPL-3.0-or-later
// Original SIG/InterIndex lifecycle observations, not a native graph parity claim.
import java.util.*;
import java.util.concurrent.atomic.AtomicInteger;
import java.nio.file.*;
import java.lang.reflect.*;
import java.awt.geom.*;
import com.fasterxml.jackson.databind.*;
import ij.process.ByteProcessor;
public class BeamExtendInParallelControlProbe extends GridProbe {
 private final Object sourceBook;
 public BeamExtendInParallelControlProbe(org.audiveris.omr.sheet.Book b,SortedSet<Integer> ids){super(b,ids);sourceBook=b;}
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
   for(JsonNode b:c.get("beams")){JsonNode m=b.get("median");raw.add(make("sig.inter.BeamInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,b.get("distance_impact").asDouble()),new Line2D.Double(m.get(0).asDouble(),m.get(1).asDouble(),m.get(2).asDouble(),m.get(3).asDouble()),b.get("height").asDouble()));}
   Object a=raw.get(0);Object side=Enum.valueOf((Class)cls("util.HorizontalSide"),c.get("side").asText());set(builder,"system",system);field(sheet,"scale",scale);JsonNode size=c.get("system_size");field(system,"area",new Area(new java.awt.Rectangle(0,0,size.get(0).asInt(),size.get(1).asInt())));Map<String,Object> row=new LinkedHashMap<>();row.put("input",c);
   byte[] supplied=((byte[])pixels.getPixels()).clone();Arrays.fill((byte[])pixels.getPixels(),(byte)0);
   Method register=builder.getClass().getDeclaredMethod("registerBeam",cls("sig.inter.AbstractBeamInter"));register.setAccessible(true);for(Object v:raw)register.invoke(builder,v);set(builder,"rawSystemBeams",raw);System.arraycopy(supplied,0,(byte[])pixels.getPixels(),0,supplied.length);
   if(c.has("pre_removed"))call(raw.get(c.get("pre_removed").asInt()-1),"remove");Object cause=Enum.valueOf((Class)cls("sig.relation.Exclusion$ExclusionCause"),"OVERLAP");for(int i=1;i<raw.size();i++)if(!(Boolean)call(raw.get(i),"isRemoved"))call(sig,"addEdge",a,raw.get(i),make("sig.relation.Exclusion",cause));
   row.put("before",snapshot(sig,index,raw));Line2D m=(Line2D)call(a,"getMedian");double half=(Double)call(scale,"toPixelsDouble",call(cls("sig.inter.BeamGroupInter"),"getMaxYDistance"));Area area=(Area)call(cls("math.AreaUtil"),"horizontalParallelogram",m.getP1(),m.getP2(),2*half);List<Object> neighbors=(List<Object>)call(cls("sig.inter.Inters"),"intersectedInters",raw,Enum.valueOf((Class)cls("math.GeoOrder"),"NONE"),area);neighbors.remove(a);List<Integer> ordered=new ArrayList<>();List<Object> attempts=new ArrayList<>();Integer selected=null;
   for(Object other:neighbors){int id=raw.indexOf(other)+1;ordered.add(id);}for(Object other:neighbors){Map<String,Object> attempt=new LinkedHashMap<>();int id=raw.indexOf(other)+1;attempt.put("candidate",id);attempt.put("gap",null);attempt.put("point",null);attempt.put("obstruction",null);String outcome="same_glyph";
    if(call(other,"getGlyph")!=call(a,"getGlyph")){outcome="not_neighbors";if((Boolean)call(cls("sig.inter.BeamGroupInter"),"canBeNeighbors",a,other,scale)){Line2D n=(Line2D)call(other,"getMedian");double x=side.toString().equals("LEFT")?n.getX1():n.getX2(),gap=side.toString().equals("LEFT")?m.getX1()-x:x-m.getX2();attempt.put("gap",gap);outcome="insufficient_increment";if(gap>=2*(Integer)BeamImpactsControlProbe.get(params,"maxStemBeamGapX")){Point2D point=(Point2D)call(cls("math.LineUtil"),"intersectionAtX",m,x);attempt.put("point",new double[]{point.getX(),point.getY()});outcome="outside_system";Method contains=builder.getClass().getDeclaredMethod("isSideInSystem",Point2D.class,double.class);contains.setAccessible(true);if((Boolean)contains.invoke(builder,point,call(a,"getHeight"))){Method lookup=builder.getClass().getDeclaredMethod("getSideBeam",cls("sig.inter.AbstractBeamInter"),cls("util.HorizontalSide"),Double.class);lookup.setAccessible(true);Object obstruction=lookup.invoke(builder,a,side,gap);attempt.put("obstruction",obstruction==null?null:raw.indexOf(obstruction)+1);outcome=obstruction==null?"point_extension":"side_obstruction";if(obstruction==null)selected=id;}}}}attempt.put("outcome",outcome);attempts.add(attempt);if(selected!=null)break;}
   row.put("ordered",ordered);row.put("attempts",attempts);row.put("attempted_point",selected);
   if(c.path("compose").asBoolean()){set(builder,"sortedSystemSeeds",new ArrayList<>());set(builder,"sortedBeamSpots",new ArrayList<>());Method lookup=builder.getClass().getDeclaredMethod("getSideBeam",cls("sig.inter.AbstractBeamInter"),cls("util.HorizontalSide"),Double.class);lookup.setAccessible(true);if(lookup.invoke(builder,a,side,null)!=null)throw new IllegalStateException("unexpected side beam");for(String name:new String[]{"extendToStem","extendToSpot"}){Method method=builder.getClass().getDeclaredMethod(name,cls("sig.inter.AbstractBeamInter"),cls("util.HorizontalSide"),Integer.class);method.setAccessible(true);if((Boolean)method.invoke(builder,a,side,null))throw new IllegalStateException("unexpected earlier acceptance");}row.put("earlier_stem_spot_rejected",true);}
   Method extend=builder.getClass().getDeclaredMethod("extendInParallel",cls("sig.inter.AbstractBeamInter"),cls("util.HorizontalSide"));extend.setAccessible(true);boolean accepted=(Boolean)extend.invoke(builder,a,side);row.put("accepted",accepted);row.put("after",snapshot(sig,index,raw));
   if(accepted){Object merged=raw.get(raw.size()-1);Line2D nm=(Line2D)call(merged,"getMedian");row.put("merged",Map.of("median",new double[]{nm.getX1(),nm.getY1(),nm.getX2(),nm.getY2()},"height",call(merged,"getHeight"),"grade",call(merged,"getGrade")));}
   for(Object v:raw)if(!(Boolean)call(v,"isRemoved"))call(v,"remove");results.add(row);
  }
  mapper.writeValue(Path.of(System.getProperty("native.probe.output")).toFile(),Map.of("original_method","BeamsBuilder.extendInParallel with real Book/Sheet/SIG", "native_outputs_used",false,"cases",results));System.out.println("original extendInParallel cases="+results.size());
 }catch(Exception e){throw new RuntimeException(e);}
 }
}
