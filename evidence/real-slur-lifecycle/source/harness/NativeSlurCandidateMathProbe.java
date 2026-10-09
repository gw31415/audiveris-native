// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.Point;
import java.awt.geom.*;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
public class NativeSlurCandidateMathProbe {
 static Class<?> c(String name)throws Exception{return Class.forName("org.audiveris.omr."+name);}
 static Object call(Object o,String name,Object...args)throws Exception {for(Class<?> k=o.getClass();k!=null;k=k.getSuperclass())for(Method m:k.getDeclaredMethods())if(m.getName().equals(name)&&m.getParameterCount()==args.length){m.setAccessible(true);try{return m.invoke(o,args);}catch(IllegalArgumentException e){}}throw new NoSuchMethodException(name);}
 static Object field(Object o,String name)throws Exception {for(Class<?> k=o.getClass();k!=null;k=k.getSuperclass())try{Field f=k.getDeclaredField(name);f.setAccessible(true);return f.get(o);}catch(NoSuchFieldException e){}throw new NoSuchFieldException(name);}
 static void set(Object o,String name,Object value)throws Exception {for(Class<?> k=o.getClass();k!=null;k=k.getSuperclass())try{Field f=k.getDeclaredField(name);f.setAccessible(true);f.set(o,value);return;}catch(NoSuchFieldException e){}throw new NoSuchFieldException(name);}
 static Map<String,Object> map(Object...a){Map<String,Object>r=new LinkedHashMap<>();for(int i=0;i<a.length;i+=2)r.put((String)a[i],a[i+1]);return r;}
 static String json(Object v){if(v==null)return "null";if(v instanceof Double d&&!Double.isFinite(d))return json(d.toString());if(v instanceof Float f&&!Float.isFinite(f))return json(f.toString());if(v instanceof Number||v instanceof Boolean)return v.toString();if(v instanceof Map<?,?>m){List<String>s=new ArrayList<>();for(var e:m.entrySet())s.add(json(e.getKey())+":"+json(e.getValue()));return "{"+String.join(",",s)+"}";}if(v instanceof Collection<?>xs){List<String>s=new ArrayList<>();for(Object x:xs)s.add(json(x));return "["+String.join(",",s)+"]";}return "\""+v.toString().replace("\\","\\\\").replace("\"","\\\"").replace("\n","\\n").replace("\r","\\r").replace("\t","\\t")+"\"";}
 static Object point(Point2D p){return p==null?null:List.of(p.getX(),p.getY());}
 static Object bezier(Object o){if(o==null)return null;CubicCurve2D b=(CubicCurve2D)o;return List.of(point(b.getP1()),point(b.getCtrlP1()),point(b.getCtrlP2()),point(b.getP2()));}
 static Map<String,Object> circle(Object o)throws Exception{return map("center",point((Point2D)call(o,"getCenter")),"radius",call(o,"getRadius"),"distance",call(o,"getDistance"),"ccw",call(o,"ccw"),"above",call(o,"isAbove"),"arc_angle",call(o,"getArcAngle"),"mid_angle",call(o,"getMidAngle"),"first_angle",call(o,"getFirstAngle"),"last_angle",call(o,"getLastAngle"));}
 static String exception(Throwable e){while(e instanceof InvocationTargetException&&e.getCause()!=null)e=e.getCause();return e.getClass().getName()+": "+e.getMessage();}
 static Object allocate(Class<?> type)throws Exception{Class<?> u=Class.forName("sun.misc.Unsafe");Field f=u.getDeclaredField("theUnsafe");f.setAccessible(true);Object unsafe=f.get(null);return u.getMethod("allocateInstance",Class.class).invoke(unsafe,type);}
 public static void main(String[]args)throws Exception {
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");Object scale=call(sheet,"getScale");
  Constructor<?> pc=c("sheet.curve.SlursBuilder$Parameters").getDeclaredConstructor(c("sheet.Scale"));pc.setAccessible(true);Object params=pc.newInstance(scale);
  // Curves constructor runs Skeleton; allocate a detached builder only for params-dependent math.
  Object builder=allocate(c("sheet.curve.SlursBuilder"));set(builder,"params",params);
  int sideLength=(Integer)field(params,"sideLength");double minGrade=(Double)c("sig.inter.SlurInter").getMethod("getMinGrade").invoke(null);
  List<Object> rows=new ArrayList<>();
  for(String line:Files.readAllLines(Path.of(args[1]))){if(line.isBlank())continue;String[]cols=line.split("\\t");List<Point>points=new ArrayList<>();List<Object> pointRows=new ArrayList<>();for(String pair:cols[2].split(";")){String[]xy=pair.split(",");Point p=new Point(Integer.parseInt(xy[0]),Integer.parseInt(xy[1]));points.add(p);pointRows.add(List.of(p.x,p.y));}
   Map<String,Object> row=map("candidate_index",Integer.parseInt(cols[0]),"kind",cols[1],"points",pointRows,"point_count",points.size());
   try{Object rough=c("math.Circle").getConstructor(Point2D.class,Point2D.class,Point2D.class).newInstance(points.get(0),points.get(points.size()/2),points.get(points.size()-1));row.put("rough_before_public_distance",circle(rough));row.put("rough_public_distance_rms",call(rough,"computeDistance",points));row.put("rough_after_public_distance",circle(rough));}catch(Throwable e){row.put("rough_exception",exception(e));}
   try{Object fitted=c("math.Circle").getConstructor(List.class).newInstance(points);row.put("fitted_before_public_distance",circle(fitted));row.put("fitted_public_distance_rms",call(fitted,"computeDistance",points));row.put("fitted_after_public_distance",circle(fitted));}catch(Throwable e){row.put("fitted_exception",exception(e));}
   try{Object model=call(builder,"computeModel",points,false);row.put("compute_model_return",model==null?null:model.getClass().getSimpleName());if(model!=null)row.put("selected_before_impacts",circle(call(model,"getCircle")));
    Object slur=c("sheet.curve.SlurInfo").getConstructor(int.class,Point.class,Point.class,List.class,c("sheet.curve.Model"),Collection.class,int.class).newInstance(Integer.parseInt(cols[0])+1,null,null,points,model,List.of(),sideLength);
    Object impacts=call(builder,"computeImpacts",slur,true);row.put("compute_impacts_return",impacts==null?null:impacts.getClass().getSimpleName());double grade=Double.NaN;
    if(impacts!=null){List<Object> entries=new ArrayList<>();int count=(Integer)call(impacts,"getImpactCount");for(int i=0;i<count;i++)entries.add(map("name",call(impacts,"getName",i),"weight",call(impacts,"getWeight",i),"clamped_impact",call(impacts,"getImpact",i)));grade=(Double)call(impacts,"getGrade");row.put("impacts",entries);row.put("grade",grade);}
    Object actualModel=call(slur,"getModel");Object left=call(slur,"getSideModel",true),right=call(slur,"getSideModel",false);row.put("side_models",map("left_class",left==null?null:left.getClass().getSimpleName(),"right_class",right==null?null:right.getClass().getSimpleName(),"left_is_global",left!=null&&left==actualModel,"right_is_global",right!=null&&right==actualModel,"left_is_right",left!=null&&left==right));
    Object curve=call(slur,"getCurve");row.put("bezier",bezier(curve));row.put("math_accepted_before_glyph_and_linkage",impacts!=null&&grade>=minGrade&&curve!=null);if(actualModel!=null)row.put("selected_after_impacts",circle(call(actualModel,"getCircle")));
   }catch(Throwable e){row.put("math_exception",exception(e));row.put("math_accepted_before_glyph_and_linkage",false);}
   rows.add(row);
  }
  Files.writeString(Path.of(args[2]),json(map("scope","Pinned original Circle constructors, detached original SlursBuilder.computeModel/computeImpacts and original SlurInfo.getCurve on identical actual native candidate point sequences. Saved original book supplies its actual Sheet Scale only. No image recognition, createInter/retrieveGlyph, SlurLinker, graph insertion or export is invoked.","builder_instrumentation","Unsafe allocates an otherwise uninitialized detached SlursBuilder, then original private Parameters(actualSheetScale) populates its params. The invoked methods use only params and supplied model/points; Curves construction (which would build Skeleton) is intentionally avoided. Not an ordinary lifecycle observation.","interline",call(scale,"getInterline"),"side_length",sideLength,"min_grade",minGrade,"rows",rows))+"\n");
 }
}
