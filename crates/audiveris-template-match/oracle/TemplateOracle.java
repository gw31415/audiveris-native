// Reflection adapter only: all distance and template calculations use the official JAR.
import java.nio.file.*;
import java.util.*;
import java.awt.geom.Point2D;
public class TemplateOracle {
  static Class<?> bp,dt,tc,ac;
  static Object call(Object o,String n)throws Exception{return o.getClass().getMethod(n).invoke(o);}
  static String array(int[] a){return Arrays.toString(a);}
  static Object table(int w,int h,int[] pixels,boolean[] mask,String storage,String target)throws Exception{
    Object b=bp.getConstructor(int.class,int.class).newInstance(w,h);
    for(int i=0;i<pixels.length;i++)bp.getMethod("set",int.class,int.class).invoke(b,i,mask[i]?255:pixels[i]);
    Class<?> c=Class.forName("org.audiveris.omr.image.ChamferDistance$"+storage);
    Object t=c.getMethod("computeTo"+target,bp).invoke(c.getConstructor().newInstance(),b);
    for(int i=0;i<mask.length;i++)if(mask[i])dt.getMethod("setValue",int.class,int.class).invoke(t,i,-1);
    return t;
  }
  static int[] values(Object t,int n)throws Exception{int[]a=new int[n];for(int i=0;i<n;i++)a[i]=(Integer)dt.getMethod("getValue",int.class).invoke(t,i);return a;}
  @SuppressWarnings({"unchecked","rawtypes"})
  public static void main(String[]args)throws Exception{
    bp=Class.forName("ij.process.ByteProcessor");dt=Class.forName("org.audiveris.omr.image.DistanceTable");
    tc=Class.forName("org.audiveris.omr.image.Template");ac=Class.forName("org.audiveris.omr.image.Anchored$Anchor");
    StringJoiner cases=new StringJoiner(",","[","]");
    for(int mode=0;mode<8;mode++)for(String storage:new String[]{"Short","Integer"})for(String target:new String[]{"Fore","Back"}){
      int w=mode==6?12000:mode==7?1:37,h=mode==6?1:mode==7?67:29,n=w*h;
      int[]p=new int[n];boolean[]mask=new boolean[n];
      for(int i=0;i<n;i++){
        p[i]=switch(mode){case 0->255;case 1->0;case 2->(i==0?0:255);case 3->(i==n-1?0:255);case 4->((i*17+i/7)%13==0?0:17);case 5->((i*17+i/7)%13==0?0:255);default->(i==0?0:255);};
        mask[i]=mode==5&&(i%11==0||i/w==14);
      }
      Object t=table(w,h,p,mask,storage,target);
      cases.add("{\"mode\":"+mode+",\"width\":"+w+",\"height\":"+h+",\"storage\":\""+storage+"\",\"target\":\""+target+"\",\"pixels\":"+array(p)+",\"mask\":"+Arrays.toString(mask)+",\"values\":"+array(values(t,n))+"}");
    }
    Class<?> fc=Class.forName("org.audiveris.omr.image.TemplateFactory"),mf=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily"),sc=Class.forName("org.audiveris.omr.glyph.Shape");
    Class.forName("org.audiveris.omr.ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
    Object f=fc.getMethod("getInstance").invoke(null),family=Enum.valueOf((Class)mf,"Bravura");
    Object catalog=fc.getMethod("getCatalog",mf,int.class).invoke(f,family,44);
    int w=64,h=48;int[]p=new int[w*h];boolean[]mask=new boolean[p.length];
    for(int i=0;i<p.length;i++){p[i]=((i*17+i/7)%13==0||(i%w>=19&&i%w<29&&i/w>=12&&i/w<17))?0:255;mask[i]=i%19==0||i/w==13;}
    Object distances=table(w,h,p,mask,"Short","Fore");
    StringJoiner templates=new StringJoiner(",","[","]");
    for(String shape:new String[]{"NOTEHEAD_BLACK","NOTEHEAD_VOID","WHOLE_NOTE"}){
      Object t=catalog.getClass().getMethod("getTemplate",sc).invoke(catalog,Enum.valueOf((Class)sc,shape));
      StringJoiner points=new StringJoiner(",","[","]"),anchors=new StringJoiner(",","[","]"),scores=new StringJoiner(",","[","]");
      for(Object point:(List<?>)call(t,"getKeyPoints"))points.add("["+point.getClass().getField("x").get(point)+","+point.getClass().getField("y").get(point)+","+point.getClass().getField("d").get(point)+"]");
      Map<?,?> offsets=(Map<?,?>)call(t,"getOffsets");
      List<Object> all=new ArrayList<>();all.add(null);all.addAll(offsets.keySet());
      for(Object anchor:all){
        String name=anchor==null?"null":"\""+anchor.toString()+"\"";
        if(anchor!=null){Point2D o=(Point2D)offsets.get(anchor),r=(Point2D)tc.getMethod("getOffset",ac).invoke(t,anchor);Object side=ac.getMethod("hSide").invoke(anchor);anchors.add("{\"name\":"+name+",\"x\":"+o.getX()+",\"y\":"+o.getY()+",\"side\":\""+(side==null?"CENTER":side)+"\",\"rounded\":["+(int)r.getX()+","+(int)r.getY()+"]}");}
        for(int[]xy:new int[][]{{0,0},{23,15},{63,47},{-10,4},{-1000,-1000}}){
          double score=(Double)tc.getMethod("evaluate",int.class,int.class,ac,dt).invoke(t,xy[0],xy[1],anchor,distances);
          double hole=(Double)tc.getMethod("evaluateHole",int.class,int.class,ac,dt).invoke(t,xy[0],xy[1],anchor,distances);
          scores.add("{\"anchor\":"+name+",\"x\":"+xy[0]+",\"y\":"+xy[1]+",\"score\":"+score+",\"hole\":"+hole+"}");
        }
      }
      templates.add("{\"shape\":\""+shape+"\",\"point_size\":44,\"width\":"+call(t,"getWidth")+",\"height\":"+call(t,"getHeight")+",\"points\":"+points+",\"anchors\":"+anchors+",\"scores\":"+scores+"}");
    }
    Files.writeString(Path.of(args[0]),"{\"java\":\""+System.getProperty("java.version")+"\",\"distance_cases\":"+cases+",\"score_input\":{\"width\":"+w+",\"height\":"+h+",\"pixels\":"+array(p)+",\"mask\":"+Arrays.toString(mask)+"},\"templates\":"+templates+"}\n");
    System.exit(0);
  }
}
