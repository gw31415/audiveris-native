// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only independent calls into original Audiveris 5.11.0 geometry.
// No production precheck or dispatch is replayed by this observer.
import java.util.*;
import java.nio.file.*;
import java.lang.reflect.*;
import java.awt.geom.Line2D;
import com.fasterxml.jackson.databind.*;
@SuppressWarnings({"unchecked", "rawtypes", "deprecation"})
public class BeamStructureControlProbe {
 static Class<?> cls(String n)throws Exception{return Class.forName("org.audiveris.omr."+n);}
 static Object make(String n,Object...args)throws Exception{for(Constructor<?> c:cls(n).getConstructors())if(c.getParameterCount()==args.length)return c.newInstance(args);throw new NoSuchMethodException(n);}
 static Object call(Object target,String name,Object...args)throws Exception{for(Method m:target.getClass().getMethods())if(m.getName().equals(name)&&m.getParameterCount()==args.length){try{return m.invoke(target,args);}catch(IllegalArgumentException e){}}throw new NoSuchMethodException(name);}
 static Object value(String n,String v)throws Exception{return cls(n).getField(v).get(null);}
 static Object get(Object target,String name)throws Exception{Field f=target.getClass().getDeclaredField(name);f.setAccessible(true);return f.get(target);}
 static void set(Object target,String name,Object value)throws Exception{Field f=target.getClass().getDeclaredField(name);f.setAccessible(true);f.set(target,value);}
 static double[] segment(Object raw){Line2D l=(Line2D)raw;return new double[]{l.getX1(),l.getY1(),l.getX2(),l.getY2()};}
 static Object lines(Object s)throws Exception{List<Object> out=new ArrayList<>();for(Object l:(List<?>)call(s,"getLines")){List<Object> items=new ArrayList<>();for(Object i:(List<?>)call(l,"getItems"))items.add(Map.of("median",segment(get(i,"median")),"height",get(i,"height")));out.add(Map.of("median",segment(get(l,"median")),"height",get(l,"height"),"items",items));}return out;}
 static Object sections(Object s)throws Exception{Method m=cls("sheet.beam.BeamStructure").getDeclaredMethod("getGlyphSections");m.setAccessible(true);List<Object> out=new ArrayList<>();for(Object v:(List<?>)m.invoke(s)){List<Object> runs=new ArrayList<>();for(Object r:(List<?>)call(v,"getRuns"))runs.add(new int[]{(Integer)call(r,"getStart"),(Integer)call(r,"getLength")});java.awt.Rectangle b=(java.awt.Rectangle)call(v,"getBounds");out.add(Map.of("first_pos",call(v,"getFirstPos"),"bounds",new int[]{b.x,b.y,b.width,b.height},"weight",call(v,"getWeight"),"runs",runs));}return out;}
 public static void main(String[] args)throws Exception{
  ObjectMapper mapper=new ObjectMapper().enable(SerializationFeature.ORDER_MAP_ENTRIES_BY_KEYS);JsonNode input=mapper.readTree(Files.readString(Path.of(args[0])));List<Object> results=new ArrayList<>();
  for(JsonNode c:input){Map<String,Object> out=new LinkedHashMap<>();out.put("case",c);try{
   JsonNode b=c.get("glyph").get("bounds");Object orientation=value("run.Orientation","VERTICAL");Object t=make("run.RunTable",orientation,b.get(2).asInt(),b.get(3).asInt());
   for(JsonNode r:c.get("glyph").get("runs"))call(t,"addRun",r.get(0).asInt(),r.get(1).asInt(),r.get(2).asInt());Object glyph=make("glyph.Glyph",b.get(0).asInt(),b.get(1).asInt(),t);
   int il=c.path("interline").asInt(16);double height=c.path("typical_height").asDouble(8.0);
   Object scale=make("sheet.Scale",make("sheet.Scale$InterlineScale",il,il,il),make("sheet.Scale$LineScale",2,2,2),null,null,null);
   Object p=make("sheet.beam.BeamsBuilder$ItemParameters",scale,height,false);
   if(c.has("parameter_overrides")){Iterator<Map.Entry<String,JsonNode>> it=c.get("parameter_overrides").fields();while(it.hasNext()){var e=it.next();if(e.getKey().equals("coreSectionWidth"))set(p,e.getKey(),e.getValue().asInt());else set(p,e.getKey(),e.getValue().asDouble());}}
   Object structure=make("sheet.beam.BeamStructure",glyph,make("lag.BasicLag","controlLag",orientation),p);
   Double distance;
   if(c.has("initial_lines")){
    List rawLines=(List)call(structure,"getLines");for(JsonNode l:c.get("initial_lines")){JsonNode a=l.get("median");Object beam=make("sheet.beam.BeamLine",new Line2D.Double(a.get(0).asDouble(),a.get(1).asDouble(),a.get(2).asDouble(),a.get(3).asDouble()),l.get("height").asDouble());rawLines.add(beam);}
    distance=0.0;out.put("scope","injected_line_geometry_methods_only_computeLines_not_called");
   }else distance=(Double)call(structure,"computeLines");out.put("sections",sections(structure));if(c.has("initial_lines")){out.put("initial_lines",lines(structure));}else{out.put("global_distance",distance);out.put("after_compute_lines",lines(structure));}
   if(distance!=null){out.put("width",call(structure,"getWidth"));out.put("slope_difference",call(structure,"compareSlopes"));call(structure,"adjustSides");out.put("after_adjust_sides",lines(structure));call(structure,"extendMiddleLines");out.put("after_extend_middle",lines(structure));call(structure,"splitLines");out.put("after_split_lines",lines(structure));List<Object> jitters=new ArrayList<>();for(Object l:(List<?>)call(structure,"getLines")){try{jitters.add(Map.of("top",call(structure,"computeJitter",l,value("util.VerticalSide","TOP")),"bottom",call(structure,"computeJitter",l,value("util.VerticalSide","BOTTOM"))));}catch(Exception ex){Throwable cause=ex instanceof InvocationTargetException?ex.getCause():ex;jitters.add(Map.of("exception",cause.getClass().getSimpleName()));}}out.put("jitters",jitters);}
  }catch(Exception e){Throwable v=e instanceof InvocationTargetException?e.getCause():e;out.put("exception",v.getClass().getSimpleName());out.put("error",v.toString());}results.add(out);}
  mapper.writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),Map.of("mode","direct_original_BeamStructure_methods_no_builder_decision_replay","cases",results));
 }
}
