// Development-only independent original BeamItem core/belt mask control.
// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.geom.*;
import java.nio.file.*;
import java.util.*;
import com.fasterxml.jackson.databind.*;
public class BeamItemGeometryReviewProbe {
 static Map<String,Object> sample(Area a){var b=a.getBounds();StringBuilder bits=new StringBuilder();for(int y=b.y;y<b.y+b.height;y++)for(int x=b.x;x<b.x+b.width;x++)bits.append(a.contains(x,y)?'1':'0');return Map.of("bounds",new int[]{b.x,b.y,b.width,b.height},"contains",bits.toString());}
 public static void main(String[] args)throws Exception{ObjectMapper m=new ObjectMapper();var root=m.readTree(Files.readString(Path.of(args[0])));Class<?> cl=Class.forName("org.audiveris.omr.sheet.beam.BeamItem");var con=cl.getConstructor(Line2D.class,double.class);var getCore=cl.getMethod("getCoreArea");var getBelt=cl.getMethod("getBeltArea",Area.class,int.class,int.class,int.class);List<Object> out=new ArrayList<>();for(var c:root){var p=c.get("median");Object item=con.newInstance(new Line2D.Double(p.get(0).asDouble(),p.get(1).asDouble(),p.get(2).asDouble(),p.get(3).asDouble()),c.get("height").asDouble());Area core=(Area)getCore.invoke(item),belt=(Area)getBelt.invoke(item,core,c.get("dx").asInt(),c.get("top").asInt(),c.get("bottom").asInt());out.add(Map.of("case",c,"core",sample(core),"belt",sample(belt)));}m.writeValue(Path.of(args[1]).toFile(),out);}
}
