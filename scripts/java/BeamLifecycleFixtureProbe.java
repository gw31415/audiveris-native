// SPDX-License-Identifier: AGPL-3.0-or-later
// Original SIG/InterIndex lifecycle observations, not a native graph parity claim.
import java.util.*;
import java.util.concurrent.atomic.AtomicInteger;
import java.nio.file.*;
import java.lang.reflect.*;
import java.awt.geom.*;
import com.fasterxml.jackson.databind.*;
import ij.process.ByteProcessor;
public class BeamLifecycleFixtureProbe extends GridProbe {
 private final Object sourceBook;
 public BeamLifecycleFixtureProbe(org.audiveris.omr.sheet.Book b,SortedSet<Integer> ids){super(b,ids);sourceBook=b;}
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
  Object scale=make("sheet.Scale",make("sheet.Scale$InterlineScale",16,16,16),make("sheet.Scale$LineScale",2,2,2),null,null,null);
  Object builder=unsafe.allocateInstance(cls("sheet.beam.BeamsBuilder"));set(builder,"stdParams",make("sheet.beam.BeamsBuilder$ItemParameters",scale,8.,false));set(builder,"params",make("sheet.beam.BeamsBuilder$Parameters",scale));set(builder,"sheet",sheet);set(builder,"sig",sig);set(builder,"sheetBox",new java.awt.Rectangle(0,0,128,96));
  ByteProcessor pixels=new ByteProcessor(128,96);Arrays.fill((byte[])pixels.getPixels(),(byte)255);for(int x=10;x<100;x++)for(int y=36;y<44;y++)pixels.set(x,y,0);set(builder,"pixelFilter",pixels);
  Method register=builder.getClass().getDeclaredMethod("registerBeam",cls("sig.inter.AbstractBeamInter"));register.setAccessible(true);
  Object a=beam(10,30),b=beam(34,70);register.invoke(builder,a);register.invoke(builder,b);
  Object glyphA=call(a,"getGlyph");call(glyphA,"addGroup",cls("glyph.GlyphGroup").getField("BEAM_SPOT").get(null));register.invoke(builder,a);if(call(a,"getGlyph")!=glyphA)throw new AssertionError("glyph identity lost");
  boolean duplicate=(Boolean)call(sig,"addVertex",a);
  Object cause=Enum.valueOf((Class)cls("sig.relation.Exclusion$ExclusionCause"),"OVERLAP");
  call(sig,"addEdge",a,b,make("sig.relation.Exclusion",cause));
  List<Object> raw=new ArrayList<>(List.of(a,b));List<Object> states=new ArrayList<>();states.add(snapshot(sig,index,raw));
  Arrays.fill((byte[])pixels.getPixels(),(byte)255);
  Method merge=builder.getClass().getDeclaredMethod("mergeOf",cls("sig.inter.AbstractBeamInter"),cls("sig.inter.AbstractBeamInter"));merge.setAccessible(true);
  Object rejected=merge.invoke(builder,a,b);Map<String,Object> afterReject=snapshot(sig,index,raw);states.add(afterReject);
  if(rejected!=null || !states.get(0).equals(afterReject))throw new AssertionError("rejection changed graph");
  for(int x=10;x<70;x++)for(int y=36;y<44;y++)pixels.set(x,y,0);
  Object merged=merge.invoke(builder,a,b);if(merged==null)throw new AssertionError("expected accepted merge");
  int unregistered=(Integer)call(merged,"getId");register.invoke(builder,merged);raw.add(merged);states.add(snapshot(sig,index,raw));
  call(a,"remove");states.add(snapshot(sig,index,raw));call(b,"remove");call(b,"remove");states.add(snapshot(sig,index,raw));
  Object extra=beam(10,30);register.invoke(builder,extra);raw.add(extra);states.add(snapshot(sig,index,raw));
  Map<String,Object> result=Map.of("scope","official fixture Book/SheetStub/Grid system; original registerBeam/glyph identity, mergeOf rejection, index and exclusion removal; no ensembles/chords","native_outputs_used",false,"duplicate_add_return",duplicate,"unregistered_merge_id",unregistered,"states",states,"new_beam_group_is_null",call(merged,"getGroup")==null);
  new ObjectMapper().writeValue(Path.of(System.getProperty("native.probe.output")).toFile(),result);System.out.println("original fixture lifecycle snapshots="+states.size());
 }catch(Exception e){throw new RuntimeException(e);}
 }
}
