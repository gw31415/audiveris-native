// SPDX-License-Identifier: AGPL-3.0-or-later
// Original SIG/InterIndex lifecycle observations, not a native graph parity claim.
import java.util.*;
import java.util.concurrent.atomic.AtomicInteger;
import java.nio.file.*;
import java.lang.reflect.*;
import java.awt.geom.*;
import com.fasterxml.jackson.databind.*;
import ij.process.ByteProcessor;
public class BeamExtendToStemControlProbe extends GridProbe {
 private final Object sourceBook;
 public BeamExtendToStemControlProbe(org.audiveris.omr.sheet.Book b,SortedSet<Integer> ids){super(b,ids);sourceBook=b;}
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
   Object a=raw.get(0),b=raw.get(1);Line2D am=(Line2D)call(a,"getMedian");Object side=Enum.valueOf((Class)cls("util.HorizontalSide"),c.get("side").asText());Object vertical=Enum.valueOf((Class)cls("run.Orientation"),"VERTICAL"),group=Enum.valueOf((Class)cls("glyph.GlyphGroup"),"VERTICAL_SEED");List<Object> seeds=new ArrayList<>(),owned=new ArrayList<>();List<Integer> ownedIds=new ArrayList<>();
   for(JsonNode seed:c.get("seeds")){JsonNode bounds=seed.get("bounds");int sw=bounds.get(2).asInt(),sh=bounds.get(3).asInt();ByteProcessor buffer=new ByteProcessor(sw,sh);Arrays.fill((byte[])buffer.getPixels(),(byte)255);for(JsonNode r:seed.get("runs"))for(int y=r.get(1).asInt();y<r.get(1).asInt()+r.get(2).asInt();y++)buffer.set(r.get(0).asInt(),y,0);Object table=call(make("run.RunTableFactory",vertical),"createTable",buffer),glyph=make("glyph.Glyph",bounds.get(0).asInt(),bounds.get(1).asInt(),table);seeds.add(glyph);if(seed.path("grouped").asBoolean(true))call(glyph,"addGroup",group);if(seed.path("owned").asBoolean(true)){owned.add(glyph);if(seed.path("grouped").asBoolean(true))ownedIds.add(seeds.size());}}
   // Preserve original free-glyph insertion order and group filtering.
   field(system,"freeGlyphs",new LinkedHashSet(owned));List<Object> grouped=(List<Object>)call(system,"getGroupedGlyphs",group);set(builder,"sortedSystemSeeds",grouped);
   Integer cap=c.has("max_extension")&&!c.get("max_extension").isNull()?c.get("max_extension").asInt():null;int reach=cap==null?(Integer)BeamImpactsControlProbe.get(params,"maxExtensionToStem"):Math.min(cap,(Integer)BeamImpactsControlProbe.get(params,"maxExtensionToStem")),margin=(Integer)BeamImpactsControlProbe.get(params,"maxStemBeamGapY");Method areaMethod=builder.getClass().getDeclaredMethod("sideAreaOf",String.class,cls("sig.inter.AbstractBeamInter"),cls("util.HorizontalSide"),double.class,double.class,double.class);areaMethod.setAccessible(true);Area area=(Area)areaMethod.invoke(builder,"|",a,side,(double)margin,(double)reach,0.);List<Object> matches=new ArrayList<>((Collection<?>)call(cls("glyph.Glyphs"),"intersectedGlyphs",grouped,area));Collections.sort(matches,(Comparator)cls("glyph.Glyphs").getField("byAbscissa").get(null));Object chosen=matches.isEmpty()?null:matches.get(side.toString().equals("LEFT")?matches.size()-1:0);List<Integer> ordered=new ArrayList<>();for(Object g:matches)ordered.add(seeds.indexOf(g)+1);Point2D point=null;double[] stemLine=null;if(chosen!=null){Point2D start=(Point2D)call(chosen,"getStartPoint",vertical),stop=(Point2D)call(chosen,"getStopPoint",vertical);stemLine=new double[]{start.getX(),start.getY(),stop.getX(),stop.getY()};point=(Point2D)call(cls("math.LineUtil"),"intersection",am,new Line2D.Double(start,stop));}
   Map<String,Object> row=new LinkedHashMap<>();row.put("input",c);row.put("reach",reach);row.put("vertical_margin",margin);row.put("owned_grouped",ownedIds);row.put("ordered",ordered);row.put("selected",chosen==null?null:seeds.indexOf(chosen)+1);row.put("point",point==null?null:new double[]{point.getX(),point.getY()});row.put("stem_line",stemLine);
   byte[] supplied=((byte[])pixels.getPixels()).clone();Arrays.fill((byte[])pixels.getPixels(),(byte)0);
   Method register=builder.getClass().getDeclaredMethod("registerBeam",cls("sig.inter.AbstractBeamInter"));register.setAccessible(true);register.invoke(builder,a);register.invoke(builder,b);set(builder,"rawSystemBeams",raw);System.arraycopy(supplied,0,(byte[])pixels.getPixels(),0,supplied.length);
   if(c.path("compose").asBoolean()){Method lookup=builder.getClass().getDeclaredMethod("getSideBeam",cls("sig.inter.AbstractBeamInter"),cls("util.HorizontalSide"),Double.class);lookup.setAccessible(true);Object neighbor=lookup.invoke(builder,a,side,null);if(neighbor==null)throw new IllegalStateException("missing side neighbor");if(side.toString().equals("LEFT")){Method merge=builder.getClass().getDeclaredMethod("extendToBeam",cls("sig.inter.AbstractBeamInter"),cls("sig.inter.AbstractBeamInter"));merge.setAccessible(true);if((Boolean)merge.invoke(builder,a,neighbor))throw new IllegalStateException("unexpected prefix merge");}Line2D bm=(Line2D)call(neighbor,"getMedian");double gap=side.toString().equals("LEFT")?am.getX1()-bm.getX2():bm.getX1()-am.getX2();Integer derived=(int)Math.rint(Math.max(0,gap-(Integer)BeamImpactsControlProbe.get(params,"beamsXMargin")));if(!derived.equals(cap))throw new IllegalStateException("prefix cap mismatch");row.put("prefix_neighbor",raw.indexOf(neighbor)+1);row.put("prefix_max_extension",derived);}
   Object cause=Enum.valueOf((Class)cls("sig.relation.Exclusion$ExclusionCause"),"OVERLAP");call(sig,"addEdge",a,b,make("sig.relation.Exclusion",cause));
   row.put("before",snapshot(sig,index,raw));
   Method extend=builder.getClass().getDeclaredMethod("extendToStem",cls("sig.inter.AbstractBeamInter"),cls("util.HorizontalSide"),Integer.class);extend.setAccessible(true);boolean accepted=(Boolean)extend.invoke(builder,a,side,cap);row.put("accepted",accepted);row.put("after",snapshot(sig,index,raw));
   if(accepted){Object merged=raw.get(2);Line2D m=(Line2D)call(merged,"getMedian");row.put("merged",Map.of("median",new double[]{m.getX1(),m.getY1(),m.getX2(),m.getY2()},"height",call(merged,"getHeight"),"grade",call(merged,"getGrade"),"kind",merged.getClass().getSimpleName()));}
   for(Object v:raw)if(!(Boolean)call(v,"isRemoved"))call(v,"remove");results.add(row);
  }
  mapper.writeValue(Path.of(System.getProperty("native.probe.output")).toFile(),Map.of("original_method","BeamsBuilder.extendToStem with real Book/Sheet/SIG", "native_outputs_used",false,"cases",results));System.out.println("original extendToStem cases="+results.size());
 }catch(Exception e){throw new RuntimeException(e);}
 }
}
