// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only: invoke unmodified original BeamsBuilder item methods with
// independent supplied rasters. Unsafe only instantiates the enclosing builder;
// Parameters, ItemParameters, BeamItem, AreaMask and impacts execute original code.
import java.util.*;
import java.nio.file.*;
import java.lang.reflect.*;
import java.awt.Rectangle;
import java.awt.geom.*;
import com.fasterxml.jackson.databind.*;
import ij.process.ByteProcessor;
@SuppressWarnings({"unchecked", "rawtypes", "deprecation"})
public class BeamImpactsControlProbe {
 static Class<?> cls(String n)throws Exception{return Class.forName("org.audiveris.omr."+n);}
 static Object make(String n,Object...args)throws Exception{for(Constructor<?> c:cls(n).getDeclaredConstructors())if(c.getParameterCount()==args.length){c.setAccessible(true);try{return c.newInstance(args);}catch(IllegalArgumentException e){}}throw new NoSuchMethodException(n);}
 static void set(Object target,String name,Object value)throws Exception{Field f=target.getClass().getDeclaredField(name);f.setAccessible(true);f.set(target,value);}
 static Object get(Object target,String name)throws Exception{Field f=target.getClass().getDeclaredField(name);f.setAccessible(true);return f.get(target);}
 static Object call(Object target,String name,Object...args)throws Exception{Class<?> type=target instanceof Class?(Class<?>)target:target.getClass();for(Method m:type.getMethods())if(m.getName().equals(name)&&m.getParameterCount()==args.length){try{return m.invoke(target instanceof Class?null:target,args);}catch(IllegalArgumentException e){}}throw new NoSuchMethodException(name);}
 static Map<String,Object> sample(Area a,ByteProcessor pixels)throws Exception{Object black=make("util.WrappedInteger",0);int total=(Integer)call(make("image.AreaMask",a),"fore",black,pixels);Rectangle b=a.getBounds();List<Object> contained=new ArrayList<>();for(int y=b.y;y<b.y+b.height;y++)for(int x=b.x;x<b.x+b.width;x++)if(a.contains(x,y))contained.add(new int[]{x,y});return Map.of("total",total,"black",black.getClass().getField("value").get(black),"points",contained);}
 public static void main(String[] args)throws Exception{
  ObjectMapper mapper=new ObjectMapper().enable(SerializationFeature.ORDER_MAP_ENTRIES_BY_KEYS);JsonNode input=mapper.readTree(Files.readString(Path.of(args[0])));List<Object> results=new ArrayList<>();
  Field uf=sun.misc.Unsafe.class.getDeclaredField("theUnsafe");uf.setAccessible(true);sun.misc.Unsafe unsafe=(sun.misc.Unsafe)uf.get(null);
  Class<?> builder=cls("sheet.beam.BeamsBuilder"), ip=cls("sheet.beam.BeamsBuilder$ItemParameters");
  Method beam=builder.getDeclaredMethod("computeBeamImpacts",cls("sheet.beam.BeamItem"),boolean.class,boolean.class,double.class,ip);beam.setAccessible(true);
  Method hook=builder.getDeclaredMethod("computeHookImpacts",cls("sheet.beam.BeamItem"),double.class,ip);hook.setAccessible(true);
  for(JsonNode c:input){Map<String,Object> out=new LinkedHashMap<>();out.put("case",c);
   int il=c.path("interline").asInt(16);double height=c.path("typical_height").asDouble(8.0);
   Object scale=make("sheet.Scale",make("sheet.Scale$InterlineScale",il,il,il),make("sheet.Scale$LineScale",2,2,2),null,null,null);
   Object p=make("sheet.beam.BeamsBuilder$ItemParameters",scale,height,c.path("small").asBoolean(false));
   Object params=make("sheet.beam.BeamsBuilder$Parameters",scale);
   if(c.has("parameter_overrides")){Iterator<Map.Entry<String,JsonNode>> it=c.get("parameter_overrides").fields();while(it.hasNext()){var e=it.next();set(params,e.getKey(),e.getValue().asDouble());}}
   ByteProcessor pixels=new ByteProcessor(c.path("image_width").asInt(160),c.path("image_height").asInt(96));Arrays.fill((byte[])pixels.getPixels(),(byte)255);
   for(JsonNode r:c.path("rectangles")){int v=r.size()>4?r.get(4).asInt():0;for(int y=r.get(1).asInt();y<r.get(1).asInt()+r.get(3).asInt();y++)for(int x=r.get(0).asInt();x<r.get(0).asInt()+r.get(2).asInt();x++)if(x>=0&&y>=0&&x<pixels.getWidth()&&y<pixels.getHeight())pixels.set(x,y,v);}
   for(JsonNode v:c.path("pixels"))pixels.set(v.get(0).asInt(),v.get(1).asInt(),v.get(2).asInt());
   JsonNode m=c.get("median");Object item=make("sheet.beam.BeamItem",new Line2D.Double(m.get(0).asDouble(),m.get(1).asDouble(),m.get(2).asDouble(),m.get(3).asDouble()),c.path("height").asDouble(height));
   boolean isHook=c.path("hook").asBoolean(false),top=isHook||c.path("top").asBoolean(true),bottom=isHook||c.path("bottom").asBoolean(true);
   int dx=(Integer)get(params,"beltMarginDx"),dy=(Integer)get(params,"beltMarginDy");Area core=(Area)call(item,"getCoreArea"),belt=(Area)call(item,"getBeltArea",core,dx,top?dy:0,bottom?dy:0);
   out.put("core",sample(core,pixels));out.put("belt",sample(belt,pixels));out.put("width",(int)Math.rint(m.get(2).asDouble()-m.get(0).asDouble()+1));out.put("belt_margins",new int[]{dx,dy});
   Object b=unsafe.allocateInstance(builder);set(b,"params",params);set(b,"pixelFilter",pixels);
   Object impacts=(isHook?hook.invoke(b,item,c.path("distance_impact").asDouble(1),p):beam.invoke(b,item,top,bottom,c.path("distance_impact").asDouble(1),p));
   out.put("impacts",null);out.put("accepted",false);
   if(impacts!=null){double[] values=new double[(Integer)call(impacts,"getImpactCount")],weights=new double[values.length];for(int i=0;i<values.length;i++){values[i]=(Double)call(impacts,"getImpact",i);weights[i]=(Double)call(impacts,"getWeight",i);}out.put("impacts",Map.of("values",values,"weights",weights,"grade",(Double)call(impacts,"getGrade")));out.put("accepted",(Double)call(impacts,"getGrade")>=(isHook?(Double)call(cls("sig.inter.BeamHookInter"),"getMinGrade"):(Double)call(cls("sig.inter.BeamInter"),"getMinGrade")));}
   results.add(out);
  }
  mapper.writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),Map.of("mode","direct_original_BeamsBuilder_item_methods_supplied_rasters_no_builder_orchestration", "min_beam_grade",(Double)call(cls("sig.inter.BeamInter"),"getMinGrade"),"min_hook_grade",(Double)call(cls("sig.inter.BeamHookInter"),"getMinGrade"),"cases",results));
 }
}
