// SPDX-License-Identifier: AGPL-3.0-or-later
// Declared numeric controls on original private VLinker, not page successes.
import java.awt.*;import java.awt.geom.*;import java.lang.reflect.*;import java.nio.file.*;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class BeamCloserLimitProbe extends BeamOriginDependencyProbe {
 static Object beam(Object sig,int id,double x1,double y,double x2,double h,double grade,boolean hook)throws Exception {
  Object imp=c("sig.inter.AbstractBeamInter$Impacts").getConstructor(double.class,double.class,double.class,double.class,double.class,double.class).newInstance(1.,1.,1.,1.,1.,1.);
  Object b=c("sig.inter."+(hook?"BeamHookInter":"BeamInter")).getConstructor(c("sig.GradeImpacts"),Line2D.class,double.class).newInstance(imp,new Line2D.Double(x1,y,x2,y),h);
  call(b,"setGrade",grade);call(sig,"addVertex",b);return b;
 }
 static Map<String,Object> declared(Object b,int id,int group)throws Exception {return obj("id",id,"group",group,"bounds",rect(call(b,"getBounds")),"median",line(call(b,"getMedian")),"height",call(b,"getHeight"),"grade",call(b,"getGrade"),"hook",b.getClass().getSimpleName().equals("BeamHookInter"));}
 public static void main(String[]args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((java.util.List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((java.util.List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
  Object r=c("sheet.stem.StemsRetriever").getConstructor(c("sheet.SystemInfo")).newInstance(sys);
  Constructor<?>ctor=c("sheet.stem.StemsRetriever$Parameters").getDeclaredConstructor(c("sheet.SystemInfo"),c("sheet.Scale"));ctor.setAccessible(true);set(r,"params",ctor.newInstance(sys,call(sheet,"getScale")));set(r,"watch",c("util.StopWatch").getConstructor(String.class).newInstance("closer-limit controls"));call(r,"inspectStems");
  Object root=((java.util.List<?>)get(r,"systemBeams")).get(0),bl=call(root,"getLinker"),b=((java.util.List<?>)get(bl,"allBLinkers")).get(0),v=((Map<?,?>)get(b,"vLinkers")).values().iterator().next();
  call(sys,"setArea",new Area(new Rectangle(0,0,1280,420)));set(bl,"beamBox",new Rectangle(450,195,100,10));set(b,"refPt",new Point2D.Double(500,200));
  java.util.List<Object>out=new ArrayList<>();
  String[]names={"empty","own-group","nearest-not-list-order","border-distance-not-median","weak-grade","good-threshold","hook-excluded","outside-segment","endpoint-included","near-side-excluded","side-dx-equality","group-dy-equality","beyond-group-dy","center-uses-right","upward-opposite-border","stable-equal-distance","fractional-nearest-border"};
  for(String name:names){double rx=name.equals("fractional-nearest-border")?500.1:500,ry=name.equals("fractional-nearest-border")?200.1:200;set(b,"refPt",new Point2D.Double(rx,ry));int dir=name.equals("upward-opposite-border")?-1:1;boolean left=!name.equals("center-uses-right");set(b,"hSide",left?c("util.HorizontalSide").getField("LEFT").get(null):null);set(v,"yDir",dir);set(v,"vSide",c("util.VerticalSide").getField(dir>0?"BOTTOM":"TOP").get(null));double[]theo={rx,ry,rx,dir>0?350:50};set(v,"theoLine",new Line2D.Double(theo[0],theo[1],theo[2],theo[3]));
   java.util.List<Object>bs=new ArrayList<>(),data=new ArrayList<>();
   if(name.equals("own-group")){Object q=beam(sig,1,450,230,550,6,.8,false);call(call(root,"getGroup"),"addMember",q);bs.add(q);data.add(declared(q,1,1));}
   else if(!name.equals("empty")){
    double x1=450,x2=550,y=230,grade=.8,h=6;boolean hook=false;
    if(name.equals("border-distance-not-median")){y=220;h=30;}
    if(name.equals("fractional-nearest-border")){x1=440.2;x2=550.7;y=230.1;h=6.1;}
    if(name.equals("weak-grade"))grade=.349999;
    if(name.equals("good-threshold"))grade=.35;
    if(name.equals("hook-excluded"))hook=true;
    if(name.equals("outside-segment"))x1=501;
    if(name.equals("endpoint-included")){x1=500;y=260;}
    if(name.equals("near-side-excluded"))x1=498;
    if(name.equals("side-dx-equality"))x1=497;
    if(name.equals("group-dy-equality")){x1=498;y=255;}
    if(name.equals("beyond-group-dy")){x1=498;y=255.01;}
    if(name.equals("center-uses-right"))x2=502;
    if(dir<0)y=170;
    Object q=beam(sig,2,x1,y,x2,h,grade,hook);bs.add(q);data.add(declared(q,2,2));
    if(name.equals("nearest-not-list-order")||name.equals("stable-equal-distance")||name.equals("border-distance-not-median")||name.equals("fractional-nearest-border")){Object z=beam(sig,3,name.equals("fractional-nearest-border")?440.3:name.equals("stable-equal-distance")?450:440,name.equals("fractional-nearest-border")?230.1:name.equals("stable-equal-distance")?y:name.equals("border-distance-not-median")?224:220,name.equals("fractional-nearest-border")?550.8:name.equals("stable-equal-distance")?560:550,name.equals("fractional-nearest-border")?6.1:6,.8,false);bs.add(z);data.add(declared(z,3,3));}
   }
   @SuppressWarnings("unchecked") Comparator<Object>byX=(Comparator<Object>)c("sig.inter.Inters").getField("byAbscissa").get(null);bs.sort(byX);set(r,"systemBeams",bs);
   Line2D result=(Line2D)call(v,"getCloserLimit");Object selection=null;
   if(result!=null){Object opp=c("util.VerticalSide").getField(dir>0?"TOP":"BOTTOM").get(null);for(Object q:bs){Line2D edge=(Line2D)call(q,"getBorder",opp);if(edge.getP1().equals(result.getP1())&&edge.getP2().equals(result.getP2())){int i=0;for(Object d:data){if(Arrays.equals((int[])((Map<?,?>)d).get("bounds"),(int[])rect(call(q,"getBounds"))))break;i++;}selection=obj("id",((Map<?,?>)data.get(i)).get("id"),"border",line(result));break;}}}
   out.add(obj("name",name,"search",obj("group",1,"beam_bounds",new int[]{450,195,100,10},"system_bounds",new int[]{0,0,1280,420},"reference",new double[]{rx,ry},"theoretical_line",theo,"direction",dir,"left",left,"interline",11),"beams",data,"selected",selection));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","17 declared numeric original getCloserLimit controls in explicitly substituted geometry; separate from full-image comparison","native_outputs_used",false,"controls",out));System.exit(0);
 }
}
