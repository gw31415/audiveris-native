public class RestNoteDurationProbe {
 public static void main(String[] args) throws Exception {
  Class<?> shape=Class.forName("org.audiveris.omr.glyph.Shape");
  String[] names={"LONG_REST","BREVE_REST","WHOLE_REST","HALF_REST","QUARTER_REST","EIGHTH_REST","ONE_16TH_REST","ONE_32ND_REST","ONE_64TH_REST","ONE_128TH_REST","NOTEHEAD_BLACK","NOTEHEAD_VOID"};
  try (var output=new java.io.PrintStream(args[0],java.nio.charset.StandardCharsets.UTF_8)) {
  output.print("{\"scope\":\"direct original Shape.getNoteDuration scalar controls\",\"cases\":[");
  boolean first=true;
  for(String name:names){Object s=shape.getMethod("valueOf",String.class).invoke(null,name);Object d=shape.getMethod("getNoteDuration").invoke(s);if(!first)output.print(",");first=false;output.print("{\"shape\":\""+name+"\",\"duration\":\""+d+"\"}");}
  output.println("]}");
  }
 }
}
