// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only direct calls into the unchanged original Audiveris 5.11 JAR.
import java.awt.Rectangle;
import java.awt.geom.Line2D;
import java.nio.file.*;
import java.util.*;
import java.lang.reflect.*;
public class HeadStemKernelProbe {
 static Object call(Object obj,String name,Object... args)throws Exception {
  Class<?> cl=obj instanceof Class<?>?(Class<?>)obj:obj.getClass();
  for(Method m:cl.getMethods())if(m.getName().equals(name)&&m.getParameterCount()==args.length){boolean ok=true;Class<?>[] ts=m.getParameterTypes();for(int i=0;i<ts.length;i++)if(args[i]!=null&&!ts[i].isInstance(args[i])&&!(ts[i]==double.class&&args[i] instanceof Double)&&!(ts[i]==boolean.class&&args[i] instanceof Boolean)&&!(ts[i]==int.class&&args[i] instanceof Integer))ok=false;if(ok)return m.invoke(obj instanceof Class<?>?null:obj,args);}
  throw new NoSuchMethodException(name);
 }
 public static void main(String[] args) throws Exception {
  Class<?> relation=Class.forName("org.audiveris.omr.sig.relation.HeadStemRelation"),headClass=Class.forName("org.audiveris.omr.sig.inter.HeadInter"),shapeClass=Class.forName("org.audiveris.omr.glyph.Shape"),staffClass=Class.forName("org.audiveris.omr.sheet.Staff");
  Constructor<?> headConstructor=headClass.getConstructor(Rectangle.class,shapeClass,Double.class,staffClass,Double.class);
  List<String> portions=new ArrayList<>(),consistency=new ArrayList<>(),invading=new ArrayList<>(),limits=new ArrayList<>();
  for(int height:new int[]{1,2,7,10,15,32})for(double[] line:new double[][]{{10,20,10,100},{10,20,12,25},{10,20,10,20},{10,100,10,20}}){
   Object head=headConstructor.newInstance(new Rectangle(5,10,15,height),shapeClass.getField("NOTEHEAD_BLACK").get(null),Double.valueOf(.6),null,Double.valueOf(0));
   double margin=height*.275,mid=(line[1]+line[3])/2.;
   for(double y:new double[]{line[1]-1,line[1]+margin-1e-9,line[1]+margin,line[1]+margin+1e-9,mid-1e-9,mid,mid+1e-9,line[3]-margin-1e-9,line[3]-margin,line[3]-margin+1e-9,line[3]+1})
    portions.add("{\"bounds\":[5,10,15,"+height+"],\"stem\":"+Arrays.toString(line)+",\"anchor_y\":"+y+",\"portion\":\""+call(relation,"getStemPortion",head,new Line2D.Double(line[0],line[1],line[2],line[3]),y)+"\"}");
  }
  for(boolean small:new boolean[]{false,true})for(double length:new double[]{0,1e-9,.5,1,2.8-1e-9,2.8,2.8+1e-9,3.5,10,100}){
   Object r=relation.getConstructor().newInstance();call(r,"setConsistency",small,length);double value=(Double)call(r,"getConsistency");
   consistency.add("{\"small\":"+small+",\"length_interlines\":"+length+",\"consistency\":"+(Double.isFinite(value)?Double.toString(value):"\""+value+"\"")+"}");
  }
  for(double dx:new double[]{-.4,-.2,-1e-9,0,.05-1e-9,.05,.05+1e-9,.15,.25,.35})for(double dy:new double[]{-.1,-1e-9,0,1e-9,.1,.4,.8}){
   Object r=relation.getConstructor().newInstance();call(r,"setInOutGaps",dx,dy,0);invading.add("{\"dx\":"+dx+",\"dy\":"+dy+",\"invading\":"+call(r,"isInvading")+"}");
  }
  for(int profile:new int[]{0,1,2})limits.add("{\"profile\":"+profile+",\"x_in_max\":"+call(call(relation,"getXInGapMaximum",profile),"getValue")+",\"x_out_max\":"+call(call(relation,"getXOutGapMaximum",profile),"getValue")+",\"y_max\":"+call(call(relation,"getYGapMaximum",profile),"getValue")+"}");
  Files.writeString(Path.of(args[0]),"{\"native_outputs_used\":false,\"scope\":\"Direct original HeadStemRelation kernels with default constants; not stem attachment or full STEMS.\",\"default_consistency\":"+call(relation.getConstructor().newInstance(),"getConsistency")+",\"portions\":["+String.join(",",portions)+"],\"consistency\":["+String.join(",",consistency)+"],\"invading\":["+String.join(",",invading)+"],\"gap_limits\":["+String.join(",",limits)+"]}\n");
 }
}
