// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only original5.11 reflection controls; no native runtime input.
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
public class ClefPitchControls {
 @SuppressWarnings({"rawtypes","unchecked"})
 public static void main(String[] args)throws Exception {
  Class<?> c=Class.forName("org.audiveris.omr.sig.inter.ClefInter"), shape=Class.forName("org.audiveris.omr.glyph.Shape");
  Constructor<?> ctor=c.getConstructor(shape,Double.class);
  Method set=c.getMethod("setPitch",Double.class),step=c.getMethod("noteStepOf",c,int.class),oct=c.getMethod("octaveOf",c,double.class);
  List<String> rows=new ArrayList<>();
  double[] pitches={-10,-5.5,-4.5,-2,-1.5,-.5,0,.5,1.5,2,4,10};
  for(String s:new String[]{"G_CLEF","G_CLEF_SMALL","G_CLEF_8VA","G_CLEF_8VB","F_CLEF","F_CLEF_SMALL","F_CLEF_8VA","F_CLEF_8VB","C_CLEF"}) {
   double[] clefPitches=s.equals("C_CLEF")?new double[]{-4,-1.5,-.5,0,.5,1.5,4}:new double[]{s.startsWith("G")?2:-2};
   for(double cp:clefPitches){Object clef=ctor.newInstance(Enum.valueOf((Class)shape,s),1.0);set.invoke(clef,cp);
    for(double p:pitches)rows.add("{\"shape\":\""+s+"\",\"clef_pitch\":"+cp+",\"head_pitch\":"+p+",\"step\":\""+step.invoke(null,clef,(int)Math.rint(p))+"\",\"octave\":"+oct.invoke(null,clef,p)+"}");
   }
  }
  Files.writeString(Path.of(args[0]),"{\"scope\":\"original_ClefInter_static_pitch_controls_not_page_recognition\",\"native_outputs_used\":false,\"cases\":["+String.join(",",rows)+"]}\n");
 }
}
