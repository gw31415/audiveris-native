// SPDX-License-Identifier: AGPL-3.0-or-later
// Original area construction and selection; supplied curved staff geometry.
import java.util.*;import java.nio.file.*;import java.lang.reflect.*;import java.awt.geom.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class MultiStaffAreaControlProbe {
 static Class<?> c(String n)throws Exception{return Class.forName("org.audiveris.omr."+n);}
 static Object empty(Class<?> c)throws Exception{Constructor<?> k=c.getDeclaredConstructor();k.setAccessible(true);return k.newInstance();}
 static void set(Object o,String name,Object v)throws Exception{Field f=o.getClass().getDeclaredField(name);f.setAccessible(true);f.set(o,v);}
 static Object get(Object o,String name)throws Exception{Field f=o.getClass().getDeclaredField(name);f.setAccessible(true);return f.get(o);}
 @SuppressWarnings("unchecked") public static void main(String[]args)throws Exception {
  Object sheet=empty(c("sheet.Sheet")),picture=empty(c("sheet.Picture")),sm=empty(c("sheet.SystemManager")),system=empty(c("sheet.SystemInfo")),scale=empty(c("sheet.Scale"));
  set(picture,"width",160);set(picture,"height",320);set(sheet,"picture",picture);set(sheet,"systemManager",sm);
  Object il=c("sheet.Scale$InterlineScale").getConstructor(int.class,int.class,int.class).newInstance(16,16,16);set(scale,"interlineScale",il);set(sheet,"scale",scale);
  set(system,"areaLeft",0);set(system,"areaRight",160);((List<Object>)get(sm,"systems")).add(system);
  List<Object> staves=(List<Object>)get(system,"staves"),geometry=new ArrayList<>();
  Class<?> lc=c("sheet.grid.LineInfo");
  for(int id=1;id<=2;id++) {
   List<Object> lines=new ArrayList<>(),defs=new ArrayList<>();
   for(int n=0;n<5;n++){
    final double base=(id==1?50:194)+16*n;final double[] xs={20,80,140},ys={base,base+4,base-1};
    Object spline=c("math.NaturalSpline").getMethod("interpolate",double[].class,double[].class).invoke(null,xs,ys);
    lines.add(Proxy.newProxyInstance(lc.getClassLoader(),new Class[]{lc},(o,m,a)->{
     switch(m.getName()){
      case "getSpline":return spline;
      case "getThickness":return 1.;
      case "getEndPoint":return new Point2D.Double(a[0].toString().equals("LEFT")?20:140,a[0].toString().equals("LEFT")?base:base-1);
      case "yAt": if(m.getReturnType()==int.class)return ((Number)spline.getClass().getMethod("yAtX",int.class).invoke(spline,((Number)a[0]).intValue())).intValue();return spline.getClass().getMethod("yAtXExt",double.class).invoke(spline,((Number)a[0]).doubleValue());
      default:throw new UnsupportedOperationException(m.getName());
     }
    }));
    defs.add(Map.of("points",List.of(new double[]{20,base},new double[]{80,base+4},new double[]{140,base-1}),"thickness",1.));
   }
   Object staff=c("sheet.Staff").getConstructor().newInstance();set(staff,"id",id);set(staff,"left",20);set(staff,"right",140);set(staff,"specificInterline",16);set(staff,"lines",lines);set(staff,"system",system);staves.add(staff);
   geometry.add(Map.of("id",id,"system_id",1,"left",20,"right",140,"specific_interline",16,"lines",defs,"ledger_lines",List.of()));
  }
  Object manager=c("sheet.StaffManager").getConstructor(c("sheet.Sheet")).newInstance(sheet);set(sheet,"staffManager",manager);
  for(Object staff:staves)c("sheet.StaffManager").getMethod("computeStaffArea",c("sheet.Staff")).invoke(manager,staff);
  List<Object> cases=new ArrayList<>();
  for(double x:new double[]{-1,0,.25,19.5,20,64,80,127,140,159.999,160,161})
   for(double y:new double[]{-1,0,.25,49,50,100,127,128,129,150,153.4,153.6,160,180,193,194,220,258,319.999,320,321}) {
    Point2D point=new Point2D.Double(x,y);
    Object closest=c("sheet.StaffManager").getMethod("getClosestStaff",Point2D.class,List.class).invoke(null,point,staves);
    Object above=c("sheet.SystemInfo").getMethod("getStaffAtOrAbove",Point2D.class).invoke(system,point);
    cases.add(Map.of("x",x,"y",y,"closest",closest==null?0:c("sheet.Staff").getMethod("getId").invoke(closest),"above",above==null?0:c("sheet.Staff").getMethod("getId").invoke(above)));
   }
  Map<String,Object> grid=Map.of("staffs",geometry,"systems",List.of(Map.of("id",1,"staff_ids",List.of(1,2),"area",Map.of("x_range",List.of(0,160),"vertical_margin",14))),"vlag",List.of(),"hlag",List.of());
  new ObjectMapper().writeValue(Path.of(args[0]).toFile(),Map.of("native_outputs_used",false,"width",160,"height",320,"grid",grid,"cases",cases,"scope","Unmodified StaffManager.computeStaffArea/getClosestStaff and SystemInfo.getStaffAtOrAbove; supplied curved equal-span two-staff column; no raster/rest acceptance claim"));
 }
}
