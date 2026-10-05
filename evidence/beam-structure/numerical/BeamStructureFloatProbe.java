// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only explanation of numerical differences; original methods only.
import java.nio.file.*;
import java.util.*;
import java.lang.reflect.*;
import com.fasterxml.jackson.databind.*;
public class BeamStructureFloatProbe extends BeamStructureControlProbe {
 public static void main(String[] args)throws Exception{
  ObjectMapper mapper=new ObjectMapper();JsonNode all=mapper.readTree(Files.readString(Path.of(args[0]))),g=all.get("replayed_beam_structure_diagnostics").get(51).get("glyph"),b=g.get("bounds");
  Object orientation=value("run.Orientation","VERTICAL"),t=make("run.RunTable",orientation,b.get(2).asInt(),b.get(3).asInt());
  for(JsonNode r:g.get("runs"))call(t,"addRun",r.get(0).asInt(),r.get(1).asInt(),r.get(2).asInt());
  Object glyph=make("glyph.Glyph",b.get(0).asInt(),b.get(1).asInt(),t);int il=all.get("scale").get("interline").asInt();
  Object scale=make("sheet.Scale",make("sheet.Scale$InterlineScale",il,il,il),make("sheet.Scale$LineScale",2,2,2),null,null,null),p=make("sheet.beam.BeamsBuilder$ItemParameters",scale,all.get("scale").get("beam_thickness").asDouble(),false);
  Object structure=make("sheet.beam.BeamStructure",glyph,make("lag.BasicLag","floatLag",orientation),p);
  Method border=cls("sheet.beam.BeamStructure").getDeclaredMethod("getBorderLines",cls("glyph.Glyph"),cls("util.VerticalSide"));border.setAccessible(true);
  Map<String,Object> out=new LinkedHashMap<>();
  for(String side:List.of("TOP","BOTTOM")){List<Object> fits=new ArrayList<>();for(Object l:(List<?>)border.invoke(structure,glyph,value("util.VerticalSide",side))){Map<String,Object> fit=new LinkedHashMap<>();fit.put("mean_distance",call(l,"getMeanDistance"));for(String f:List.of("n","sx","sy","sx2","sy2","sxy","a","b","c"))fit.put(f,get(l,f));
    double n=((Number)get(l,"n")).doubleValue(),xx=(Double)get(l,"sx2"),sx=(Double)get(l,"sx"),sy=(Double)get(l,"sy"),xy=(Double)get(l,"sxy");double a=(n*xy-sx*sy)/(n*xx-sx*sx);fit.put("raw_a",a);fit.put("norm",Math.hypot(a,-1));fits.add(fit);}out.put(side,fits);}
  out.put("global_distance",call(structure,"computeLines"));mapper.writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),out);
 }
}
