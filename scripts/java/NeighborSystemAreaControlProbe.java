// SPDX-License-Identifier: AGPL-3.0-or-later
// Actual original horizontalNeighbor and computeSystemArea; no native expectations.
import java.util.*;import java.nio.file.*;import java.lang.reflect.*;import java.awt.*;import java.awt.geom.*;import com.fasterxml.jackson.databind.*;
@SuppressWarnings("unchecked")
public class NeighborSystemAreaControlProbe extends MultiStaffAreaControlProbe {
 public static void main(String[] args)throws Exception {
 ObjectMapper mapper=new ObjectMapper();JsonNode inputs=mapper.readTree(Path.of(args[0]).toFile());java.util.List<Object> configurations=new ArrayList<>();
 for(JsonNode input:inputs.get("configurations")){JsonNode grid=input.get("grid");
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

 Object manager=c("sheet.StaffManager").getConstructor(c("sheet.Sheet")).newInstance(sheet);set(sheet,"staffManager",manager);java.util.List<Object> result=new ArrayList<>();
 java.util.List<Object> staffResult=new ArrayList<>();
 for(var entry:systems.entrySet()){
 Object sys=entry.getValue();c("sheet.SystemManager").getMethod("computeSystemArea",c("sheet.SystemInfo")).invoke(sm,sys);
 java.util.List<?> north=(java.util.List<?>)c("sheet.SystemManager").getMethod("verticalNeighbors",c("sheet.SystemInfo"),c("util.VerticalSide")).invoke(sm,sys,Enum.valueOf((Class)c("util.VerticalSide"),"TOP")),south=(java.util.List<?>)c("sheet.SystemManager").getMethod("verticalNeighbors",c("sheet.SystemInfo"),c("util.VerticalSide")).invoke(sm,sys,Enum.valueOf((Class)c("util.VerticalSide"),"BOTTOM"));
 Map<String,Object> row=new LinkedHashMap<>();row.put("id",entry.getKey());row.put("north",ids(north));row.put("south",ids(south));row.put("north_staff_ids",edgeStaffs(north,false));row.put("south_staff_ids",edgeStaffs(south,true));row.put("x_range",new int[]{(int)get(sys,"areaLeft"),(int)get(sys,"areaRight")});row.put("cases",cases((Area)get(sys,"area"),width,height));result.add(row);
 }
 for(Object staff:(java.util.List<?>)get(manager,"staves")){
 c("sheet.StaffManager").getMethod("computeStaffArea",c("sheet.Staff")).invoke(manager,staff);Map<String,Object> row=new LinkedHashMap<>();row.put("id",get(staff,"id"));
 for(String side:new String[]{"LEFT","RIGHT"}){Object n=c("sheet.StaffManager").getMethod("horiNeighbor",c("sheet.Staff"),c("util.HorizontalSide")).invoke(manager,staff,Enum.valueOf((Class)c("util.HorizontalSide"),side));row.put(side.toLowerCase(),n==null?null:get(n,"id"));}
 for(String side:new String[]{"TOP","BOTTOM"}){java.util.List<?> n=(java.util.List<?>)c("sheet.StaffManager").getMethod("vertNeighbors",c("sheet.Staff"),c("util.VerticalSide")).invoke(manager,staff,Enum.valueOf((Class)c("util.VerticalSide"),side));row.put(side.equals("TOP")?"north":"south",ids(n));}
 row.put("cases",cases((Area)get(staff,"area"),width,height));staffResult.add(row);
 }
 configurations.add(Map.of("name",input.get("name").asText(),"input",input,"systems",result,"staffs",staffResult));
 }
 mapper.writeValue(Path.of(args[1]).toFile(),Map.of("native_outputs_used",false,"configurations",configurations,"scope","Unmodified original system/staff horizontal and vertical neighbors, global spline boundary areas, clip intersections and contains; synthetic horizontal layouts plus independently observed original Laura GRID; native short-staff raster discovery not claimed"));
 }
 static java.util.List<Object> ids(java.util.List<?> nodes)throws Exception {java.util.List<Object> ids=new ArrayList<>();for(Object n:nodes)ids.add(get(n,"id"));return ids;}
 static java.util.List<Object> edgeStaffs(java.util.List<?> nodes,boolean first)throws Exception {java.util.List<Object> ids=new ArrayList<>();for(Object n:nodes){java.util.List<?> s=(java.util.List<?>)get(n,"staves");ids.add(get(s.get(first?0:s.size()-1),"id"));}return ids;}
 static java.util.List<Object> cases(Area a,int width,int height){java.util.List<Object> out=new ArrayList<>();for(double x:new double[]{-1,0,80,200,350,650,width-1,width})for(double y:new double[]{0,40,60,150,187.75,188,188.25,220,430,600,800,height-1,height})out.add(Map.of("x",x,"y",y,"contains",a.contains(x,y)));return out;}

}
