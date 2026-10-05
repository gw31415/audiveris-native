// SPDX-License-Identifier: AGPL-3.0-or-later
// Original SIG/InterIndex lifecycle observations, not a native graph parity claim.
import java.util.*;
import java.util.concurrent.atomic.AtomicInteger;
import java.nio.file.*;
import java.lang.reflect.*;
import java.awt.geom.*;
import com.fasterxml.jackson.databind.*;
import ij.process.ByteProcessor;
public class BeamExtendToSpotControlProbe extends GridProbe {
 private final Object sourceBook;
 public BeamExtendToSpotControlProbe(org.audiveris.omr.sheet.Book b,SortedSet<Integer> ids){super(b,ids);sourceBook=b;}
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
   for(String key:new String[]{"first","peer"}){JsonNode b=c.get(key),m=b.get("median");raw.add(make("sig.inter.BeamInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,b.get("distance_impact").asDouble()),new Line2D.Double(m.get(0).asDouble(),m.get(1).asDouble(),m.get(2).asDouble(),m.get(3).asDouble()),b.get("height").asDouble()));}
   Object a=raw.get(0),b=raw.get(1);Line2D am=(Line2D)call(a,"getMedian");Object side=Enum.valueOf((Class)cls("util.HorizontalSide"),c.get("side").asText());
   byte[] supplied=((byte[])pixels.getPixels()).clone();Arrays.fill((byte[])pixels.getPixels(),(byte)0);
   Method register=builder.getClass().getDeclaredMethod("registerBeam",cls("sig.inter.AbstractBeamInter"));register.setAccessible(true);register.invoke(builder,a);register.invoke(builder,b);set(builder,"rawSystemBeams",raw);System.arraycopy(supplied,0,(byte[])pixels.getPixels(),0,supplied.length);
   Object horizontal=Enum.valueOf((Class)cls("run.Orientation"),"HORIZONTAL"),group=Enum.valueOf((Class)cls("glyph.GlyphGroup"),"BEAM_SPOT");List<Object> spots=new ArrayList<>(),owned=new ArrayList<>(),assigned=new ArrayList<>();
   for(JsonNode spot:c.get("spots")){Object glyph;if(spot.path("own").asBoolean()){glyph=call(a,"getGlyph");}else{JsonNode bounds=spot.get("bounds");int sw=bounds.get(2).asInt(),sh=bounds.get(3).asInt();ByteProcessor buffer=new ByteProcessor(sw,sh);Arrays.fill((byte[])buffer.getPixels(),(byte)255);for(JsonNode r:spot.get("runs"))for(int x=r.get(1).asInt();x<r.get(1).asInt()+r.get(2).asInt();x++)buffer.set(x,r.get(0).asInt(),0);glyph=make("glyph.Glyph",bounds.get(0).asInt(),bounds.get(1).asInt(),call(make("run.RunTableFactory",horizontal),"createTable",buffer));call(glyph,"setId",spots.size()+1);}spots.add(glyph);if(spot.path("grouped").asBoolean(true))call(glyph,"addGroup",group);if(spot.path("owned").asBoolean(true))owned.add(glyph);if(spot.path("assigned").asBoolean())assigned.add(glyph);}
   field(system,"freeGlyphs",new LinkedHashSet(owned));List<Object> pool=(List<Object>)call(system,"getGroupedGlyphs",group);Collections.sort(pool,(Comparator)cls("glyph.Glyphs").getField("byFullOrdinate").get(null));pool.removeAll(assigned);set(builder,"sortedBeamSpots",pool);set(builder,"sortedSystemSeeds",new ArrayList<>());
   if(c.has("seeds")){Object vertical=Enum.valueOf((Class)cls("run.Orientation"),"VERTICAL");List<Object> seeds=new ArrayList<>();for(JsonNode rawSeed:c.get("seeds")){JsonNode bounds=rawSeed.get("bounds");ByteProcessor buffer=new ByteProcessor(bounds.get(2).asInt(),bounds.get(3).asInt());Arrays.fill((byte[])buffer.getPixels(),(byte)255);for(JsonNode r:rawSeed.get("runs"))for(int y=r.get(1).asInt();y<r.get(1).asInt()+r.get(2).asInt();y++)buffer.set(r.get(0).asInt(),y,0);seeds.add(make("glyph.Glyph",bounds.get(0).asInt(),bounds.get(1).asInt(),call(make("run.RunTableFactory",vertical),"createTable",buffer)));}set(builder,"sortedSystemSeeds",seeds);}
   Integer cap=c.has("max_extension")&&!c.get("max_extension").isNull()?c.get("max_extension").asInt():null;int reach=cap==null?(Integer)BeamImpactsControlProbe.get(params,"maxExtensionToSpot"):Math.min(cap,(Integer)BeamImpactsControlProbe.get(params,"maxExtensionToSpot"));Method areaMethod=builder.getClass().getDeclaredMethod("sideAreaOf",String.class,cls("sig.inter.AbstractBeamInter"),cls("util.HorizontalSide"),double.class,double.class,double.class);areaMethod.setAccessible(true);Area area=(Area)areaMethod.invoke(builder,"O",a,side,0.,(double)reach,0.);List<Object> matches=new ArrayList<>((Collection<?>)call(cls("glyph.Glyphs"),"intersectedGlyphs",pool,area));Collections.sort(matches,(Comparator)cls("glyph.Glyphs").getField("byAbscissa").get(null));matches.remove(call(a,"getGlyph"));Object chosen=matches.isEmpty()?null:matches.get(side.toString().equals("LEFT")?matches.size()-1:0);List<Integer> ordered=new ArrayList<>(),poolIds=new ArrayList<>();for(Object g:matches)ordered.add(spots.indexOf(g)+1);for(Object g:pool)poolIds.add(spots.indexOf(g)+1);Point2D point=null;if(chosen!=null){java.awt.Rectangle bounds=(java.awt.Rectangle)call(chosen,"getBounds");point=(Point2D)call(cls("math.LineUtil"),"intersectionAtX",am,(double)(side.toString().equals("LEFT")?bounds.x:bounds.x+bounds.width-1));}
   Map<String,Object> row=new LinkedHashMap<>();row.put("input",c);row.put("pool",poolIds);row.put("reach",reach);row.put("ordered",ordered);row.put("selected",chosen==null?null:spots.indexOf(chosen)+1);row.put("point",point==null?null:new double[]{point.getX(),point.getY()});
   Object cause=Enum.valueOf((Class)cls("sig.relation.Exclusion$ExclusionCause"),"OVERLAP");call(sig,"addEdge",a,b,make("sig.relation.Exclusion",cause));
   row.put("before",snapshot(sig,index,raw));
   if(c.path("compose").asBoolean()){Method lookup=builder.getClass().getDeclaredMethod("getSideBeam",cls("sig.inter.AbstractBeamInter"),cls("util.HorizontalSide"),Double.class);lookup.setAccessible(true);Object neighbor=lookup.invoke(builder,a,side,null);Integer derived=null;if(neighbor!=null){if(side.toString().equals("LEFT")){Method merge=builder.getClass().getDeclaredMethod("extendToBeam",cls("sig.inter.AbstractBeamInter"),cls("sig.inter.AbstractBeamInter"));merge.setAccessible(true);if((Boolean)merge.invoke(builder,a,neighbor))throw new IllegalStateException("unexpected merge");}Line2D bm=(Line2D)call(neighbor,"getMedian");double gap=side.toString().equals("LEFT")?am.getX1()-bm.getX2():bm.getX1()-am.getX2();derived=(int)Math.rint(Math.max(0,gap-(Integer)BeamImpactsControlProbe.get(params,"beamsXMargin")));}if(!Objects.equals(derived,cap))throw new IllegalStateException("prefix cap mismatch");Method stem=builder.getClass().getDeclaredMethod("extendToStem",cls("sig.inter.AbstractBeamInter"),cls("util.HorizontalSide"),Integer.class);stem.setAccessible(true);row.put("preceding_stem_accepted",stem.invoke(builder,a,side,derived));if(Boolean.TRUE.equals(row.get("preceding_stem_accepted"))!=c.path("stem_success").asBoolean())throw new IllegalStateException("unexpected stem outcome");}
   Method extend=builder.getClass().getDeclaredMethod("extendToSpot",cls("sig.inter.AbstractBeamInter"),cls("util.HorizontalSide"),Integer.class);extend.setAccessible(true);boolean skipped=Boolean.TRUE.equals(row.get("preceding_stem_accepted"));row.put("spot_method_called",!skipped);boolean accepted=skipped||(Boolean)extend.invoke(builder,a,side,cap);row.put("accepted",accepted);row.put("after",snapshot(sig,index,raw));
   if(accepted){Object merged=raw.get(2);Line2D m=(Line2D)call(merged,"getMedian");row.put("merged",Map.of("median",new double[]{m.getX1(),m.getY1(),m.getX2(),m.getY2()},"height",call(merged,"getHeight"),"grade",call(merged,"getGrade"),"kind",merged.getClass().getSimpleName()));}
   for(Object v:raw)if(!(Boolean)call(v,"isRemoved"))call(v,"remove");results.add(row);
  }
  mapper.writeValue(Path.of(System.getProperty("native.probe.output")).toFile(),Map.of("original_method","Original extendToSpot with owning unassigned pool; controlled original prefix/stem preceding branch when compose=true; real Book/Sheet/SIG", "native_outputs_used",false,"cases",results));System.out.println("original extendToSpot cases="+results.size());
 }catch(Exception e){throw new RuntimeException(e);}
 }
}
