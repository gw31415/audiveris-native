// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.geom.*;import java.awt.*;import java.lang.reflect.*;import java.nio.file.*;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class BeamWidthProbe {
 public static void main(String[]a)throws Exception{
  Class<?> beam=Class.forName("org.audiveris.omr.sig.inter.BeamInter"),gi=Class.forName("org.audiveris.omr.sig.GradeImpacts");
  Object impacts=Class.forName("org.audiveris.omr.sig.inter.AbstractBeamInter$Impacts").getConstructor(double.class,double.class,double.class,double.class,double.class,double.class).newInstance(1.,1.,1.,1.,1.,1.);
  double[][] lines={{10.1,20.,192.2,20.},{10.1,30.,192.5,30.},{10.1,40.,193.5,40.}};
  java.util.List<Object>bs=new ArrayList<>(),controls=new ArrayList<>();
  for(int i=0;i<lines.length;i++){double[]l=lines[i];Object b=beam.getConstructor(gi,Line2D.class,double.class).newInstance(impacts,new Line2D.Double(l[0],l[1],l[2],l[3]),6.);bs.add(b);Rectangle r=(Rectangle)beam.getMethod("getBounds").invoke(b);controls.add(Map.of("id",i+1,"median",l,"bounds",new int[]{r.x,r.y,r.width,r.height}));}
  @SuppressWarnings("unchecked") Comparator<Object>order=(Comparator<Object>)Class.forName("org.audiveris.omr.sig.inter.Inters").getField("byReverseWidth").get(null);
  java.util.List<Object>sorted=new ArrayList<>(bs);sorted.sort(order);java.util.List<Integer>ids=new ArrayList<>();for(Object b:sorted)ids.add(bs.indexOf(b)+1);
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(a[0]).toFile(),Map.of("scope","three declared numeric actual BeamInter bounds / Inters.byReverseWidth controls; not a whole-image comparison","native_outputs_used",false,"beams",controls,"sorted_ids",ids));System.exit(0);
 }
}
