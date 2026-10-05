// SPDX-License-Identifier: AGPL-3.0-or-later
// Original SIG/InterIndex lifecycle observations, not a native graph parity claim.
import java.util.*;
import java.util.concurrent.atomic.AtomicInteger;
import java.nio.file.*;
import java.lang.reflect.*;
import java.awt.geom.*;
import com.fasterxml.jackson.databind.*;
import ij.process.ByteProcessor;
public class BeamLifecycleControlProbe extends BeamImpactsControlProbe {
 static void field(Object target,String name,Object value)throws Exception {
  for(Class<?> c=target.getClass();c!=null;c=c.getSuperclass())try{Field f=c.getDeclaredField(name);f.setAccessible(true);f.set(target,value);return;}catch(NoSuchFieldException e){}
  throw new NoSuchFieldException(name);
 }
 static Object beam(double x1,double x2)throws Exception{return make("sig.inter.BeamInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),new Line2D.Double(x1,40,x2,40),8.);}
 static Map<String,Object> snapshot(Object sig,Object index,List<Object> raw)throws Exception {
  List<Object> live=new ArrayList<>(),ids=new ArrayList<>(),removed=new ArrayList<>(),indexed=new ArrayList<>();
  for(Object v:(Collection<?>)call(sig,"vertexSet"))live.add(call(v,"getId"));
  for(Object v:raw){int id=(Integer)call(v,"getId");ids.add(id);removed.add(call(v,"isRemoved"));indexed.add(call(index,"getEntity",id)!=null);}
  return Map.of("active_ids",live,"raw_ids",ids,"removed",removed,"index_present",indexed,"edge_count",((Collection<?>)call(sig,"edgeSet")).size());
 }
 @SuppressWarnings({"unchecked","rawtypes"})
 public static void main(String[] args)throws Exception {
  Field uf=sun.misc.Unsafe.class.getDeclaredField("theUnsafe");uf.setAccessible(true);sun.misc.Unsafe unsafe=(sun.misc.Unsafe)uf.get(null);
  Object sheet=unsafe.allocateInstance(cls("sheet.Sheet")),system=unsafe.allocateInstance(cls("sheet.SystemInfo")),index=make("sig.InterIndex");
  field(index,"lastId",new AtomicInteger(0));field(sheet,"interIndex",index);field(system,"sheet",sheet);
  Object sig=make("sig.SIGraph",system);field(system,"sig",sig);
  Object a=beam(10,30),b=beam(34,70);call(sig,"addVertex",a);call(sig,"addVertex",b);
  boolean duplicate=(Boolean)call(sig,"addVertex",a);
  Object cause=Enum.valueOf((Class)cls("sig.relation.Exclusion$ExclusionCause"),"OVERLAP");
  call(sig,"addEdge",a,b,make("sig.relation.Exclusion",cause));
  List<Object> raw=new ArrayList<>(List.of(a,b));List<Object> states=new ArrayList<>();states.add(snapshot(sig,index,raw));
  Object scale=make("sheet.Scale",make("sheet.Scale$InterlineScale",16,16,16),make("sheet.Scale$LineScale",2,2,2),null,null,null);
  Object builder=unsafe.allocateInstance(cls("sheet.beam.BeamsBuilder"));set(builder,"stdParams",make("sheet.beam.BeamsBuilder$ItemParameters",scale,8.,false));set(builder,"params",make("sheet.beam.BeamsBuilder$Parameters",scale));
  ByteProcessor pixels=new ByteProcessor(128,96);Arrays.fill((byte[])pixels.getPixels(),(byte)255);set(builder,"pixelFilter",pixels);
  Method merge=builder.getClass().getDeclaredMethod("mergeOf",cls("sig.inter.AbstractBeamInter"),cls("sig.inter.AbstractBeamInter"));merge.setAccessible(true);
  Object rejected=merge.invoke(builder,a,b);Map<String,Object> afterReject=snapshot(sig,index,raw);states.add(afterReject);
  if(rejected!=null || !states.get(0).equals(afterReject))throw new AssertionError("rejection changed graph");
  for(int x=10;x<70;x++)for(int y=36;y<44;y++)pixels.set(x,y,0);
  Object merged=merge.invoke(builder,a,b);if(merged==null)throw new AssertionError("expected accepted merge");
  int unregistered=(Integer)call(merged,"getId");call(sig,"addVertex",merged);raw.add(merged);states.add(snapshot(sig,index,raw));
  call(a,"remove");states.add(snapshot(sig,index,raw));call(b,"remove");call(b,"remove");states.add(snapshot(sig,index,raw));
  Object extra=beam(80,100);call(sig,"addVertex",extra);raw.add(extra);states.add(snapshot(sig,index,raw));
  Map<String,Object> result=Map.of("scope","original SIG.addVertex and Inter.remove with initial exclusion only; registerBeam glyph retrieval and ensembles/chords not exercised","native_outputs_used",false,"duplicate_add_return",duplicate,"unregistered_merge_id",unregistered,"states",states,"new_beam_group_is_null",call(merged,"getGroup")==null);
  new ObjectMapper().writeValue(Path.of(args[0]).toFile(),result);System.out.println("original lifecycle snapshots="+states.size());
 }
}
