// Development-only independent OpenJDK line-only CAG review control.
// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.geom.*;
import java.nio.file.*;
import java.util.*;
import com.fasterxml.jackson.databind.*;
public class LinearAreaReviewProbe {
 static Area area(JsonNode points){Path2D.Double p=new Path2D.Double(Path2D.WIND_NON_ZERO);p.moveTo(points.get(0).get(0).asDouble(),points.get(0).get(1).asDouble());for(int i=1;i<points.size();i++)p.lineTo(points.get(i).get(0).asDouble(),points.get(i).get(1).asDouble());p.closePath();return new Area(p);}
 static Map<String,Object> sample(Area a,JsonNode queries){var b=a.getBounds();StringBuilder bits=new StringBuilder();for(var q:queries)bits.append(a.contains(q.get(0).asDouble(),q.get(1).asDouble())?'1':'0');var bounds2d=a.getBounds2D(); List<Object> segments=new ArrayList<>(); for(var it=a.getPathIterator(null);!it.isDone();it.next()){double[] xy=new double[6];int kind=it.currentSegment(xy);segments.add(Map.of("kind",kind,"point",new double[]{xy[0],xy[1]}));}return Map.of("bounds",new int[]{b.x,b.y,b.width,b.height},"bounds2d",new double[]{bounds2d.getX(),bounds2d.getY(),bounds2d.getWidth(),bounds2d.getHeight()},"segments",segments,"contains",bits.toString());}
 public static void main(String[] args)throws Exception{ObjectMapper m=new ObjectMapper();var root=m.readTree(Files.readString(Path.of(args[0])));List<Object> out=new ArrayList<>();for(var c:root){Area left=area(c.get("left")),right=area(c.get("right"));Map<String,Object> result=new LinkedHashMap<>();result.put("case",c);result.put("left",sample(left,c.get("queries")));result.put("right",sample(right,c.get("queries")));left.subtract(right);result.put("difference",sample(left,c.get("queries")));out.add(result);}m.writeValue(Path.of(args[1]).toFile(),out);}
}
