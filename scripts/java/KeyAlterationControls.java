// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only original5.11 KeyInter.getAlterFor kernel observations.
public class KeyAlterationControls {
 public static void main(String[] args)throws Exception {
  Class<?> step=Class.forName("org.audiveris.omr.sig.inter.AbstractNoteInter$NoteStep");
  java.lang.reflect.Method alter=Class.forName("org.audiveris.omr.sig.inter.KeyInter").getMethod("getAlterFor",step,int.class);
  System.out.print("{\"scope\":\"original KeyInter.getAlterFor explicit fifths/step kernels, separate from image comparison\",\"cases\":[");
  boolean first=true;
  for(int f=-7;f<=7;f++)for(Object s:step.getEnumConstants()){
   if(!first)System.out.print(",");first=false;
   System.out.print("{\"fifths\":"+f+",\"step\":\""+((Enum<?>)s).name()+"\",\"alter\":"+alter.invoke(null,s,f)+"}");
  }
  System.out.println("]}");
 }
}
