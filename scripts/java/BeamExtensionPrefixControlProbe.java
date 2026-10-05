// SPDX-License-Identifier: AGPL-3.0-or-later
// Original SIG/InterIndex lifecycle observations, not a native graph parity claim.
import java.util.*;
import java.util.concurrent.atomic.AtomicInteger;
import java.nio.file.*;
import java.lang.reflect.*;
import java.awt.geom.*;
import com.fasterxml.jackson.databind.*;
import ij.process.ByteProcessor;
public class BeamExtensionPrefixControlProbe extends GridProbe {
 private final Object sourceBook;
 public BeamExtensionPrefixControlProbe(org.audiveris.omr.sheet.Book b,SortedSet<Integer> ids){super(b,ids);sourceBook=b;}
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
   int il=16,w=160,h=96;Object scale=make("sheet.Scale",make("sheet.Scale$InterlineScale",il,il,il),make("sheet.Scale$LineScale",2,2,2),null,null,null);
   Object builder=unsafe.allocateInstance(cls("sheet.beam.BeamsBuilder"));set(builder,"stdParams",make("sheet.beam.BeamsBuilder$ItemParameters",scale,8.,false));Object params=make("sheet.beam.BeamsBuilder$Parameters",scale);set(builder,"params",params);set(builder,"sheet",sheet);set(builder,"sig",sig);set(builder,"sheetBox",new java.awt.Rectangle(0,0,w,h));
   ByteProcessor pixels=new ByteProcessor(w,h);Arrays.fill((byte[])pixels.getPixels(),(byte)0);set(builder,"pixelFilter",pixels);
   Method register=builder.getClass().getDeclaredMethod("registerBeam",cls("sig.inter.AbstractBeamInter"));register.setAccessible(true);
   List<Object> all=new ArrayList<>();for(JsonNode b:c.get("beams")){JsonNode m=b.get("median");Object v=make("sig.inter.BeamInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),new Line2D.Double(m.get(0).asDouble(),m.get(1).asDouble(),m.get(2).asDouble(),m.get(3).asDouble()),8.);register.invoke(builder,v);all.add(v);}
   if(c.has("pre_removed"))call(all.get(c.get("pre_removed").asInt()-1),"remove");
   Arrays.fill((byte[])pixels.getPixels(),(byte)255);for(JsonNode p:c.get("black_pixels"))pixels.set(p.get(0).asInt(),p.get(1).asInt(),0);
   List<Object> raw=new ArrayList<>((List<Object>)call(sig,"inters",cls("sig.inter.AbstractBeamInter")));set(builder,"rawSystemBeams",raw);List<Object> initial=new ArrayList<>(raw);List<Integer> initialIds=new ArrayList<>();for(Object v:initial)initialIds.add(all.indexOf(v)+1);
   List<String> sides=new ArrayList<>();for(Object v:(Object[])call(cls("util.HorizontalSide"),"values"))sides.add(v.toString());
   Method lookup=builder.getClass().getDeclaredMethod("getSideBeam",cls("sig.inter.AbstractBeamInter"),cls("util.HorizontalSide"),Double.class);lookup.setAccessible(true);Method extend=builder.getClass().getDeclaredMethod("extendToBeam",cls("sig.inter.AbstractBeamInter"),cls("sig.inter.AbstractBeamInter"));extend.setAccessible(true);
   List<Object> decisions=new ArrayList<>();List<Integer> skipped=new ArrayList<>();boolean stopped=false;
   List<Object> visits=c.get("mode").asText().equals("prefix")?initial:List.of(all.get(c.get("target").asInt()-1));
   for(Object target:visits){int id=all.indexOf(target)+1;if((Boolean)call(target,"isRemoved")){skipped.add(id);continue;}
    String side=c.get("mode").asText().equals("prefix")?sides.get(0):c.get("side").asText();Object hs=Enum.valueOf((Class)cls("util.HorizontalSide"),side);Object neighbor=lookup.invoke(builder,target,hs,null);Map<String,Object> decision=new LinkedHashMap<>();decision.put("beam",id);decision.put("side",side);decision.put("neighbor",neighbor==null?null:all.indexOf(neighbor)+1);
    boolean merged=neighbor!=null&&side.equals("LEFT")&&(Boolean)extend.invoke(builder,target,neighbor);decision.put("merged",merged);
    if(merged){Object appended=raw.get(raw.size()-1);all.add(appended);decision.put("appended",all.size());}else{Integer max=null;if(neighbor!=null){Line2D a=(Line2D)call(target,"getMedian"),b=(Line2D)call(neighbor,"getMedian");double gap=side.equals("LEFT")?a.getX1()-b.getX2():b.getX1()-a.getX2();max=(int)Math.rint(Math.max(0,gap-(Integer)BeamImpactsControlProbe.get(params,"beamsXMargin")));}decision.put("max_extension",max);decision.put("parallel_eligible_after_stem_and_spot",neighbor==null);stopped=true;}
    decisions.add(decision);if(stopped)break;
   }
   results.add(Map.of("input",c,"initial_ids",initialIds,"side_order",sides,"decisions",decisions,"skipped_removed",skipped,"stopped_for_downstream",stopped,"after",snapshot(sig,index,all)));
   for(Object v:all)if(!(Boolean)call(v,"isRemoved"))call(v,"remove");
  }
  mapper.writeValue(Path.of(System.getProperty("native.probe.output")).toFile(),Map.of("original_method","Retained extendBeams prefix order; original SIG.inters, HorizontalSide.values, getSideBeam and extendToBeam; stops before downstream branches", "native_outputs_used",false,"cases",results));System.out.println("original extendToBeam cases="+results.size());
 }catch(Exception e){throw new RuntimeException(e);}
 }
}
