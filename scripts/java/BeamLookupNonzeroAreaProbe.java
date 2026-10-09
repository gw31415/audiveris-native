// SPDX-License-Identifier: AGPL-3.0-or-later
// Declared-input kernel: pinned BeamLinker.VLinker.buildLuArea arithmetic with
// actual original LineUtil, followed by actual original-runtime Java Area.
// Does not invoke VLinker on a recognized owner or certify image ownership.
import com.fasterxml.jackson.databind.*;
import com.fasterxml.jackson.databind.node.*;
import java.awt.Rectangle;
import java.awt.geom.*;
import java.nio.file.*;
import java.util.*;
public class BeamLookupNonzeroAreaProbe {
 static final ObjectMapper M=new ObjectMapper();
 static Point2D intersectionAtX(Line2D l,double x)throws Exception{return (Point2D)Class.forName("org.audiveris.omr.math.LineUtil").getMethod("intersectionAtX",Line2D.class,double.class).invoke(null,l,x);}
 static double yAtX(Line2D l,double x)throws Exception{return (Double)Class.forName("org.audiveris.omr.math.LineUtil").getMethod("yAtX",Line2D.class,double.class).invoke(null,l,x);}
 static Point2D intersection(Line2D a,Line2D b)throws Exception{return (Point2D)Class.forName("org.audiveris.omr.math.LineUtil").getMethod("intersection",Line2D.class,Line2D.class).invoke(null,a,b);}
 static double n(JsonNode v,String key){return v.get(key).doubleValue();}
 static double[] vector(JsonNode a){double[] out=new double[a.size()];for(int i=0;i<a.size();i++)out[i]=a.get(i).doubleValue();return out;}
 static Line2D line(JsonNode a){double[] v=vector(a);return new Line2D.Double(v[0],v[1],v[2],v[3]);}
 static double[] xy(Point2D p){return new double[]{p.getX(),p.getY()};}
 static List<double[]> polygon(JsonNode r,ObjectNode result)throws Exception{
  double[] ref=vector(r.get("reference"));double x=ref[0],y=ref[1],dir=n(r,"direction");
  double slope=-n(r,"slope"),margin=dir*n(r,"slope_margin"),dx=n(r,"half_lookup_dx");
  Line2D border=line(r.get("border"));Point2D pl=intersectionAtX(border,x-dx),pr=intersectionAtX(border,x+dx);
  double offset=dir*n(r,"max_seed_dy_ratio")*n(r,"gap_pixels"),limit;
  if(r.hasNonNull("limit"))limit=yAtX(line(r.get("limit")),x);
  else{double[] box=vector(r.get("system_bounds"));limit=dir<0?box[1]+box[3]:box[1];
   for(JsonNode part:r.get("surrounding_part_bounds")){double[] b=vector(part);limit=dir>0?Math.max(limit,b[1]+b[3]-1):Math.min(limit,b[1]);}}
  double dy=limit-y;List<double[]> p=List.of(new double[]{pl.getX(),pl.getY()+offset},new double[]{pr.getX(),pr.getY()+offset},new double[]{pr.getX()+(slope+margin)*dy,limit},new double[]{pl.getX()+(slope-margin)*dy,limit});
  Point2D target=intersection(new Line2D.Double(x,y,x+100*slope,y+100),new Line2D.Double(0,limit,100,limit));
  result.set("theoretical_line",M.valueToTree(new double[]{x,y,target.getX(),target.getY()}));return p;
 }
 static List<Object> path(Area a){List<Object> out=new ArrayList<>();PathIterator i=a.getPathIterator(null);while(!i.isDone()){double[] v=new double[6];int type=i.currentSegment(v),len=type==0||type==1?2:type==2?4:type==3?6:0;out.add(Map.of("type",type,"coordinates",Arrays.copyOf(v,len)));i.next();}return out;}
 static void around(List<double[]> points,double x,double y){
  points.add(new double[]{x,y});points.add(new double[]{Math.nextDown(x),y});points.add(new double[]{Math.nextUp(x),y});points.add(new double[]{x,Math.nextDown(y)});points.add(new double[]{x,Math.nextUp(y)});
  points.add(new double[]{x-1e-7,y-1e-7});points.add(new double[]{x+1e-7,y+1e-7});
 }
 static ObjectNode observe(JsonNode c)throws Exception{
  ObjectNode row=M.createObjectNode();row.put("name",c.path("name").asText("declared-request"));List<double[]> p=new ArrayList<>();
  if(c.has("request")){row.set("request",c.get("request"));p=polygon(c.get("request"),row);row.put("kind","source-backed-declared-buildLuArea-kernel");}
  else{for(JsonNode v:c.get("polygon"))p.add(vector(v));row.put("kind","declared-Java-Area-boundary-control");}
  row.set("polygon",M.valueToTree(p));Path2D raw=new Path2D.Double();raw.moveTo(p.get(0)[0],p.get(0)[1]);for(int i=1;i<p.size();i++)raw.lineTo(p.get(i)[0],p.get(i)[1]);raw.closePath();Area area=new Area(raw);Rectangle b=area.getBounds();Rectangle2D b2=area.getBounds2D();
  row.put("path_winding_rule",raw.getWindingRule());row.put("empty",area.isEmpty());row.set("bounds",M.valueToTree(new int[]{b.x,b.y,b.width,b.height}));row.set("bounds2D",M.valueToTree(new double[]{b2.getX(),b2.getY(),b2.getWidth(),b2.getHeight()}));row.set("normalized_area_path",M.valueToTree(path(area)));
  List<double[]> points=new ArrayList<>();for(int i=0;i<p.size();i++){double[] a=p.get(i),z=p.get((i+1)%p.size());around(points,a[0],a[1]);around(points,(a[0]+z[0])/2,(a[1]+z[1])/2);}
  around(points,b2.getCenterX(),b2.getCenterY());if(c.has("points"))for(JsonNode v:c.get("points"))points.add(vector(v));
  List<Object> pointRows=new ArrayList<>();for(double[] q:points)pointRows.add(Map.of("point",q,"contains",area.contains(q[0],q[1])));row.set("points",M.valueToTree(pointRows));
  List<double[]> rectangles=new ArrayList<>();for(double[] q:p){rectangles.add(new double[]{q[0],q[1],1,1});rectangles.add(new double[]{q[0]-1,q[1]-1,1,1});}
  double x=b2.getX(),y=b2.getY(),w=b2.getWidth(),h=b2.getHeight();rectangles.addAll(List.of(new double[]{x,y,w,h},new double[]{x-1,y,1,h},new double[]{x+w,y,1,h},new double[]{x,y-1,w,1},new double[]{x,y+h,w,1},new double[]{x,y,0,1},new double[]{x,y,1,0},new double[]{x,y,-1,1}));if(c.has("rectangles"))for(JsonNode v:c.get("rectangles"))rectangles.add(vector(v));
  List<Object> rectangleRows=new ArrayList<>();for(double[] q:rectangles)rectangleRows.add(Map.of("rectangle",q,"intersects",area.intersects(q[0],q[1],q[2],q[3])));row.set("rectangles",M.valueToTree(rectangleRows));return row;
 }
 public static void main(String[] args)throws Exception{
  JsonNode input=M.readTree(Path.of(args[0]).toFile());ArrayNode rows=M.createArrayNode();for(JsonNode c:input.get("cases"))rows.add(observe(c));
  ObjectNode result=M.createObjectNode();result.put("scope","Declared-input buildLuArea kernel uses pinned original LineUtil and official runtime Area; existing 72 actual VLinker controls supply independent prior method evidence. This does not invoke VLinker or run image recognition.");result.put("original_VLinker_invoked",false);result.put("native_expected_outputs_used",false);result.put("native_geometry_may_be_supplied_as_declared_input",true);result.put("java_runtime_version",System.getProperty("java.runtime.version"));result.set("cases",rows);M.writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),result);
 }
}
