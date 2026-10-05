// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent supplied geometry; executes original getSideBeam and Inters query.
import java.util.*;
import java.nio.file.*;
import java.lang.reflect.*;
import java.awt.Rectangle;
import java.awt.geom.*;
import com.fasterxml.jackson.databind.*;
public class BeamSideLookupControlProbe extends BeamImpactsControlProbe {
 static Object beam(double[] m,double h,boolean removed)throws Exception {
  Object b=make("sig.inter.BeamInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),new Line2D.Double(m[0],m[1],m[2],m[3]),h);
  Field f=cls("sig.inter.AbstractInter").getDeclaredField("removed");f.setAccessible(true);f.set(b,removed);return b;
 }
 @SuppressWarnings({"unchecked","rawtypes"})
 public static void main(String[] args)throws Exception {
  Field uf=sun.misc.Unsafe.class.getDeclaredField("theUnsafe");uf.setAccessible(true);sun.misc.Unsafe u=(sun.misc.Unsafe)uf.get(null);
  Class<?> bt=cls("sheet.beam.BeamsBuilder"),ab=cls("sig.inter.AbstractBeamInter"),st=cls("util.HorizontalSide");
  Method lookup=bt.getDeclaredMethod("getSideBeam",ab,st,Double.class);lookup.setAccessible(true);
  Method sideArea=bt.getDeclaredMethod("sideAreaOf",String.class,ab,st,double.class,double.class,double.class);sideArea.setAccessible(true);
  List<Object> cases=new ArrayList<>();Random random=new Random(20261003);
  for(int n=0;n<300;n++) {
   int il=new int[]{8,12,16,21,32}[n%5];String side=n%2==0?"LEFT":"RIGHT";double slope=new double[]{0,.125,-.125,.065,-.065}[n%5];
   double[] median={100,80,140,80+40*slope};double height=new double[]{2,4,8}[n%3];
   Object scale=make("sheet.Scale",make("sheet.Scale$InterlineScale",il,il,il),make("sheet.Scale$LineScale",2,2,2),null,null,null),params=make("sheet.beam.BeamsBuilder$Parameters",scale);
   Object builder=u.allocateInstance(bt);set(builder,"params",params);Object target=beam(median,height,false);List<Object> raw=new ArrayList<>();raw.add(target);
   Double reach=n%8==0?null:new double[]{0,0,.5,1,Math.nextUp(1.),4,16,64}[n%8];
   for(int j=0;j<9;j++) {
    double gap=new double[]{-4,0,1,4,4,16,64,65,128}[j];
    double near=side.equals("LEFT")?100-gap:140+gap;double x1=side.equals("LEFT")?near-24:near,x2=x1+24;
    double dy=n<60?0:(j==3?il*.25:(j==4?Math.nextUp(il*.25):random.nextInt(15)-7));
    double ds=n<60?0:new double[]{0,.065,Math.nextUp(.065),-.065,-Math.nextUp(.065),.5,0,0,0}[j];
    double y1=80+(x1-100)*slope+dy,y2=y1+24*(slope+ds);
    raw.add(beam(new double[]{x1,y1,x2,y2},height,n%4==0&&j==0));
   }
   if(n%3==0)Collections.reverse(raw);
   set(builder,"rawSystemBeams",raw);Object hs=Enum.valueOf((Class)st,side);
   double usedReach=reach==null?((Integer)get(params,"maxSideBeamDx")).doubleValue():reach;
   Area area=(Area)sideArea.invoke(builder,null,target,hs,0.,usedReach,0.);
   List<?> intersects=(List<?>)call(cls("sig.inter.Inters"),"intersectedInters",raw,Enum.valueOf((Class)cls("math.GeoOrder"),"NONE"),area);
   Object result=lookup.invoke(builder,target,hs,reach);List<Integer> ordered=new ArrayList<>();List<Object> remaining=new ArrayList<>(raw);set(builder,"rawSystemBeams",remaining);
   for(int k=0;k<raw.size();k++){Object next=lookup.invoke(builder,target,hs,reach);if(next==null)break;ordered.add(raw.indexOf(next)+1);remaining.remove(next);}set(builder,"rawSystemBeams",raw);
   List<Object> inputs=new ArrayList<>();List<Integer> intersectionIds=new ArrayList<>();
   for(int k=0;k<raw.size();k++) {Object b=raw.get(k);Line2D m=(Line2D)call(b,"getMedian");Rectangle rect=(Rectangle)call(b,"getBounds");inputs.add(Map.of("id",k+1,"median",new double[]{m.getX1(),m.getY1(),m.getX2(),m.getY2()},"height",call(b,"getHeight"),"removed",call(b,"isRemoved"),"bounds",new int[]{rect.x,rect.y,rect.width,rect.height}));if(intersects.contains(b))intersectionIds.add(k+1);}
   Map<String,Object> row=new LinkedHashMap<>();row.put("interline",il);row.put("side",side);row.put("reach",reach);row.put("target",raw.indexOf(target)+1);row.put("raw",inputs);row.put("intersected",intersectionIds);row.put("selected",result==null?null:raw.indexOf(result)+1);row.put("ordered",ordered);row.put("max_slope_difference",call(cls("sig.inter.BeamGroupInter"),"getMaxSlopeDiff"));row.put("max_y_gap",get(params,"maxBeamsGapY"));row.put("default_reach",get(params,"maxSideBeamDx"));cases.add(row);
  }
  new ObjectMapper().writeValue(Path.of(args[0]).toFile(),Map.of("native_outputs_used",false,"method","BeamsBuilder.getSideBeam / Inters.intersectedInters(GeoOrder.NONE)","cases",cases));System.out.println("original side lookup cases="+cases.size());
 }
}
