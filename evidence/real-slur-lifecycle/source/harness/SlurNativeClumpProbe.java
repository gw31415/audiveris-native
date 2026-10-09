// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.Point;import java.awt.geom.*;import java.lang.reflect.*;import java.nio.file.*;import java.util.*;
public class SlurNativeClumpProbe extends SlurOwnedLookupProbeV2 {
 static Map<Object,String> names=new IdentityHashMap<>();
 static Object makeSlur(String name,List<Point>pts,double[]p,boolean above)throws Exception{
  Object s=c("sig.inter.SlurInter").getConstructor(boolean.class,Double.class).newInstance(above,1.);
  Object info=c("sheet.curve.SlurInfo").getConstructor(int.class,Point.class,Point.class,List.class,c("sheet.curve.Model"),Collection.class,int.class).newInstance(0,null,null,pts,null,List.of(),6);
  set(s,"info",info);set(s,"above",above);set(s,"curve",new CubicCurve2D.Double(p[0],p[1],p[2],p[3],p[4],p[5],p[6],p[7]));names.put(s,name);return s;
 }
 static Object entry(Object e)throws Exception{if(e==null)return null;Object s=field(e,"slur");Map<?,?>ls=(Map<?,?>)field(e,"links");return map("candidate",names.get(s),"point_count",((List<?>)call(call(s,"getInfo"),"getPoints")).size(),"mean_euclidean",call(e,"meanEuclidianDist"),"weighted_distance",call(e,"weightedDist"),"links",Arrays.asList(link(ls.get(sides(0))),link(ls.get(sides(1)))));}
 static Object newEntry(String name,int count,Object lh,double ld,Object rh,double rd)throws Exception{
  List<Point>pts=new ArrayList<>();for(int i=0;i<count;i++)pts.add(new Point(i,0));Object s=makeSlur(name,pts,new double[]{0,0,1,1,2,1,3,0},false);Map<Object,Object>links=sideMap(null,null);Object[]heads={lh,rh};double[]dist={ld,rd};
  for(int i=0;i<2;i++)if(heads[i]!=null){Object rel=c("sig.relation.SlurHeadRelation").getConstructor(c("util.HorizontalSide")).newInstance(sides(i));call(rel,"setEuclidean",dist[i]);Object l=c("sheet.curve.SlurHeadLink").getConstructor(c("sig.inter.HeadInter"),c("sig.relation.SlurHeadRelation")).newInstance(heads[i],rel);links.put(sides(i),l);}
  Constructor<?>ctor=c("sheet.curve.ClumpPruner$SlurEntry").getDeclaredConstructors()[0];ctor.setAccessible(true);return ctor.newInstance(s,links);
 }
 static Object control(String name,Object linker,List<Object>entries)throws Exception{
  List<Object>input=new ArrayList<>();for(Object e:entries)input.add(entry(e));List<?>non=(List<?>)call(linker,"getNonOrphans",entries);List<Object>chosenPool=new ArrayList<>(non.isEmpty()?entries:non);
  Object selected=call(linker,"selectAmong",chosenPool);List<Object>sorted=new ArrayList<>();for(Object e:chosenPool)sorted.add(names.get(field(e,"slur")));
  return map("case",name,"input",input,"nonorphan_count",non.size(),"sorted_pool",sorted,"selected",entry(selected),"selected_is_input_object",selected==null||entries.stream().anyMatch(e->e==selected));
 }
 public static void main(String[]args)throws Exception{
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");Object pruner=c("sheet.curve.ClumpPruner").getConstructor(c("sheet.Sheet")).newInstance(sheet);Object linker=field(pruner,"slurLinker"),manager=call(sheet,"getSystemManager");
  Map<String,List<Point>>points=new LinkedHashMap<>();for(String line:Files.readAllLines(Path.of(args[2]))){String[]f=line.split("\\t");List<Point>pts=new ArrayList<>();for(String xy:f[2].split(";")){String[]v=xy.split(",");pts.add(new Point(Integer.parseInt(v[0]),Integer.parseInt(v[1])));}points.put(f[0],pts);}
  Map<String,Object>slurs=new LinkedHashMap<>();for(String line:Files.readAllLines(Path.of(args[1]))){String[]f=line.split("\\t");String[]v=f[3].split(",");double[]p=new double[8];for(int i=0;i<8;i++)p[i]=Double.parseDouble(v[i]);slurs.put(f[0],makeSlur(f[0],points.get(f[0]),p,Boolean.parseBoolean(f[1])));}
  Constructor<?>lc=c("sheet.curve.ClumpPruner$ClumpLinker").getDeclaredConstructors()[0];lc.setAccessible(true);List<Object>rows=new ArrayList<>();Object anyLinker=null,leftHead=null,rightHead=null,otherHead=null;
  for(String line:Files.readAllLines(Path.of(args[3]))){String[]f=line.split("\t");Set<Object>clump=new LinkedHashSet<>();Map<Object,Object>areas=new LinkedHashMap<>();for(String name:f[1].split(",")){Object s=slurs.get(name);clump.add(s);areas.put(s,call(linker,"defineAreaPair",s));}
   Map<?,?>bounds=(Map<?,?>)call(pruner,"getBounds",clump,areas);List<Object>systems=new ArrayList<>((List<?>)call(manager,"getSystemsOf",bounds.get(sides(0)),null));systems.retainAll((List<?>)call(manager,"getSystemsOf",bounds.get(sides(1)),null));List<Object>attempts=new ArrayList<>();
   for(Object sys:systems){List<Object>chords=new ArrayList<>();for(Object n:(Set<?>)call(call(sys,"getSig"),"vertexSet"))if(c("sig.inter.HeadChordInter").isInstance(n))chords.add(n);Object groupLinker=lc.newInstance(pruner,sys,clump,bounds,chords);anyLinker=groupLinker;
    List<Object>input=new ArrayList<>();Map<?,?>filtered=(Map<?,?>)field(groupLinker,"chords");for(Object s:clump){Object pair=call(linker,"lookupLinkPair",s,areas.get(s),sys,filtered);Object e=null;if(pair!=null){Constructor<?>ec=c("sheet.curve.ClumpPruner$SlurEntry").getDeclaredConstructors()[0];ec.setAccessible(true);e=ec.newInstance(s,pair);Map<?,?>m=(Map<?,?>)pair;if(leftHead==null&&m.get(sides(0))!=null&&m.get(sides(1))!=null){leftHead=field(m.get(sides(0)),"partner");rightHead=field(m.get(sides(1)),"partner");}else if(otherHead==null&&m.get(sides(0))!=null&&call(field(m.get(sides(0)),"partner"),"getChord")!=call(leftHead,"getChord"))otherHead=field(m.get(sides(0)),"partner");}input.add(map("candidate",names.get(s),"entry",entry(e)));}
    Object selected=call(groupLinker,"selectSlur",areas);attempts.add(map("system",id(sys),"bounds",Arrays.asList(box((java.awt.Rectangle)bounds.get(sides(0))),box((java.awt.Rectangle)bounds.get(sides(1)))),"inputs",input,"selected",entry(selected)));
    if(selected!=null)break;
   }
   rows.add(map("group",Integer.parseInt(f[0]),"candidates",Arrays.asList(f[1].split(",")),"attempts",attempts));
  }
  List<Object>controls=new ArrayList<>();controls.add(control("empty",anyLinker,List.of()));
  Object orphan=newEntry("orphan_low",100,leftHead,.01,null,0),paired=newEntry("paired_high",3,leftHead,9,rightHead,12);
  controls.add(control("nonorphan_priority",anyLinker,List.of(orphan,paired)));controls.add(control("singleton_orphan",anyLinker,List.of(orphan)));
  Object shortE=newEntry("short",5,leftHead,4,rightHead,6),longE=newEntry("long",10,leftHead,4,rightHead,6);controls.add(control("actual_point_count_weight",anyLinker,List.of(shortE,longE)));
  Object eqA=newEntry("equal_a",5,leftHead,4,rightHead,6),eqB=newEntry("equal_b",10,leftHead,8,rightHead,12);controls.add(control("equal_stable_ab",anyLinker,List.of(eqA,eqB)));controls.add(control("equal_stable_ba",anyLinker,List.of(eqB,eqA)));
  Object diff=newEntry("different_chord_equal",5,otherHead,4,rightHead,6);controls.add(control("different_actual_chord_equal",anyLinker,List.of(eqA,diff,eqB)));
  controls.add(control("left_and_right_orphan_mean",anyLinker,List.of(newEntry("left_only",4,leftHead,8,null,0),newEntry("right_only",4,null,0,rightHead,6))));
  Files.writeString(Path.of(args[4]),json(map("scope","Actual original ClumpLinker.selectSlur plus private SlurEntry/selectAmong controls. Native component-sharing groups and native full-precision curves/actual points are inputs; this is not original Skeleton clump provenance. Existing saved Book supplies actual original HeadChord owners. No prune/doLink/tie/obstacle/cleanup callbacks, Slur insertion, recognition or Book save.","groups",rows,"controls",controls))+"\n");
 }
}
