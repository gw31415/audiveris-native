// SPDX-License-Identifier: AGPL-3.0-or-later
// Calls original mergeOf; inputs are independently supplied scenes, never Rust outputs.
import java.util.*;
import java.nio.file.*;
import java.lang.reflect.*;
import java.awt.geom.*;
import com.fasterxml.jackson.databind.*;
import ij.process.ByteProcessor;
public class BeamMergeControlProbe extends BeamImpactsControlProbe {
 public static void main(String[] args)throws Exception {
  ObjectMapper mapper=new ObjectMapper(); JsonNode input=mapper.readTree(Files.readString(Path.of(args[0])));
  Field uf=sun.misc.Unsafe.class.getDeclaredField("theUnsafe");uf.setAccessible(true);sun.misc.Unsafe unsafe=(sun.misc.Unsafe)uf.get(null);
  Class<?> builder=cls("sheet.beam.BeamsBuilder"),beam=cls("sig.inter.AbstractBeamInter");
  Method merge=builder.getDeclaredMethod("mergeOf",beam,beam);merge.setAccessible(true);
  List<Object> results=new ArrayList<>();int accepted=0,smallAccepted=0;
  for(JsonNode c:input) {
   int il=c.get("interline").asInt();
   Object scale=make("sheet.Scale",make("sheet.Scale$InterlineScale",il,il,il),make("sheet.Scale$LineScale",2,2,2),null,null,null);
   Object std=make("sheet.beam.BeamsBuilder$ItemParameters",scale,c.get("standard_height").asDouble(),false);
   Object small=c.get("small_height").isNull()?null:make("sheet.beam.BeamsBuilder$ItemParameters",scale,c.get("small_height").asDouble(),true);
   int w=c.get("width").asInt(),h=c.get("height").asInt();ByteProcessor pixels=new ByteProcessor(w,h);Arrays.fill((byte[])pixels.getPixels(),(byte)255);
   for(JsonNode p:c.get("black_pixels"))pixels.set(p.get(0).asInt(),p.get(1).asInt(),0);
   Object[] beams=new Object[2];int index=0;
   for(String key:new String[]{"first","second"}) {
    JsonNode b=c.get(key),m=b.get("median");Object impacts=make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,b.get("distance_impact").asDouble());
    beams[index++]=make(b.get("small_class").asBoolean()?"sig.inter.SmallBeamInter":"sig.inter.BeamInter",impacts,new Line2D.Double(m.get(0).asDouble(),m.get(1).asDouble(),m.get(2).asDouble(),m.get(3).asDouble()),b.get("height").asDouble());
   }
   Object target=unsafe.allocateInstance(builder);set(target,"params",make("sheet.beam.BeamsBuilder$Parameters",scale));set(target,"stdParams",std);set(target,"smallParams",small);set(target,"pixelFilter",pixels);
   Object result=merge.invoke(target,beams[0],beams[1]);Map<String,Object> row=new LinkedHashMap<>();row.put("input",c);row.put("result",null);
   if(result!=null) {
    accepted++;Line2D median=(Line2D)call(result,"getMedian");Object impacts=call(result,"getImpacts");double[] values=new double[6];for(int i=0;i<6;i++)values[i]=(Double)call(impacts,"getImpact",i);
    boolean isSmall=(Boolean)call(result,"isSmall");if(isSmall)smallAccepted++;
    row.put("result",Map.of("median",new double[]{median.getX1(),median.getY1(),median.getX2(),median.getY2()},"height",call(result,"getHeight"),"is_small",isSmall,"impacts",values,"grade",call(impacts,"getGrade")));
   }
   results.add(row);
  }
  mapper.writeValue(Path.of(args[1]).toFile(),Map.of("original_method","BeamsBuilder.mergeOf","native_outputs_used",false,"cases",results,"accepted",accepted,"small_accepted",smallAccepted));
  System.out.println("original merge cases="+results.size()+" accepted="+accepted+" small="+smallAccepted+" rejected="+(results.size()-accepted));
 }
}
