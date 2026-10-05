// SPDX-License-Identifier: AGPL-3.0-or-later
// Supplied independently observed scanned-score GRID curves; unmodified original areas.
import java.util.*;import java.nio.file.*;import java.lang.reflect.*;import java.awt.*;import java.awt.geom.*;import com.fasterxml.jackson.databind.*;
@SuppressWarnings("unchecked")
public class MultiSystemAreaControlProbe extends MultiStaffAreaControlProbe {
 public static void main(String[]args)throws Exception {
  ObjectMapper mapper=new ObjectMapper();JsonNode input=mapper.readTree(Path.of(args[0]).toFile()),grid=input.get("grid");
  Object sheet=empty(c("sheet.Sheet")),picture=empty(c("sheet.Picture")),sm=empty(c("sheet.SystemManager")),scale=empty(c("sheet.Scale"));
  int width=input.get("width").asInt(),height=input.get("height").asInt();
  set(picture,"width",width);set(picture,"height",height);set(sheet,"picture",picture);set(sheet,"systemManager",sm);set(sm,"sheet",sheet);
  set(scale,"interlineScale",c("sheet.Scale$InterlineScale").getConstructor(int.class,int.class,int.class).newInstance(11,11,11));set(sheet,"scale",scale);
  Map<Integer,Object> systems=new LinkedHashMap<>();
  for(JsonNode def:grid.get("systems")){
   Object sys=empty(c("sheet.SystemInfo"));set(sys,"id",def.get("id").asInt());set(sys,"sheet",sheet);set(sys,"areaLeft",0);set(sys,"areaRight",width);
   JsonNode box=def.get("coordinates");set(sys,"left",box.get(0).asInt());set(sys,"top",box.get(1).asInt());set(sys,"width",box.get(2).asInt());set(sys,"bottom",box.get(3).asInt());
   systems.put(def.get("id").asInt(),sys);((java.util.List<Object>)get(sm,"systems")).add(sys);
  }
  Class<?> lc=c("sheet.grid.LineInfo");
  for(JsonNode def:grid.get("staffs")){
   java.util.List<Object> lines=new ArrayList<>();
   for(JsonNode l:def.get("lines")){
    double[] xs=new double[l.get("points").size()],ys=new double[xs.length];for(int n=0;n<xs.length;n++){xs[n]=l.get("points").get(n).get(0).asDouble();ys[n]=l.get("points").get(n).get(1).asDouble();}
    Object spline=c("math.NaturalSpline").getMethod("interpolate",double[].class,double[].class).invoke(null,xs,ys);
    lines.add(Proxy.newProxyInstance(lc.getClassLoader(),new Class[]{lc},(o,m,a)->{
     switch(m.getName()){
      case "getSpline":return spline;
      case "getThickness":return l.get("thickness").asDouble();
      case "getEndPoint":return a[0].toString().equals("LEFT")?new Point2D.Double(xs[0],ys[0]):new Point2D.Double(xs[xs.length-1],ys[ys.length-1]);
      case "yAt":if(m.getReturnType()==int.class)return spline.getClass().getMethod("yAtX",int.class).invoke(spline,((Number)a[0]).intValue());return spline.getClass().getMethod("yAtXExt",double.class).invoke(spline,((Number)a[0]).doubleValue());
      default:throw new UnsupportedOperationException(m.getName());
     }
    }));
   }
   Object staff=c("sheet.Staff").getConstructor().newInstance(),sys=systems.get(def.get("system_id").asInt());set(staff,"id",def.get("id").asInt());set(staff,"left",def.get("left").asInt());set(staff,"right",def.get("right").asInt());set(staff,"specificInterline",11);set(staff,"lines",lines);set(staff,"system",sys);((java.util.List<Object>)get(sys,"staves")).add(staff);
  }
  Object manager=c("sheet.StaffManager").getConstructor(c("sheet.Sheet")).newInstance(sheet);set(sheet,"staffManager",manager);
  for(Object sys:systems.values())for(Object staff:(java.util.List<?>)get(sys,"staves"))c("sheet.StaffManager").getMethod("computeStaffArea",c("sheet.Staff")).invoke(manager,staff);
  java.util.List<Object> result=new ArrayList<>();
  for(var entry:systems.entrySet()){
   Object sys=entry.getValue();c("sheet.SystemManager").getMethod("computeSystemArea",c("sheet.SystemInfo")).invoke(sm,sys);Rectangle bounds=(Rectangle)c("sheet.SystemInfo").getMethod("getBounds").invoke(sys);
   java.util.List<Object> cases=new ArrayList<>(),sectionIds=new ArrayList<>();
   for(double x:new double[]{-1,0,72,234,500,1415,1416,width-1,width})for(double y:new double[]{0,390,391,436,515,540,637,670,692,800,875,885,886,930,1000,1200,height-1,height}) {
    Point2D p=new Point2D.Double(x,y);Object closest=c("sheet.SystemInfo").getMethod("getClosestStaff",Point2D.class).invoke(sys,p),above=c("sheet.SystemInfo").getMethod("getStaffAtOrAbove",Point2D.class).invoke(sys,p);
    cases.add(Map.of("x",x,"y",y,"closest",closest==null?0:c("sheet.Staff").getMethod("getId").invoke(closest),"above",above==null?0:c("sheet.Staff").getMethod("getId").invoke(above)));
   }
   for(JsonNode section:grid.get("vlag")){JsonNode run=section.get("runs").get(0);if(bounds.intersects(new Rectangle(section.get("first_pos").asInt(),run.get(0).asInt(),1,run.get(1).asInt())))sectionIds.add(section.get("id").asInt());}
   result.add(Map.of("id",entry.getKey(),"bounds",new int[]{bounds.x,bounds.y,bounds.width,bounds.height},"cases",cases,"section_ids",sectionIds));
  }
  mapper.writeValue(Path.of(args[1]).toFile(),Map.of("native_outputs_used",false,"input",input,"systems",result,"scope","Unmodified StaffManager/SystemManager areas and SystemInfo selection, independently observed original Laura GRID curves; supplied section rectangles; not positive raster rest recognition"));
 }
}
