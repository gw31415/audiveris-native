// Read-only reflection adapter. All connected-component code is official Audiveris.
import java.awt.Point;
import java.awt.image.BufferedImage;
import java.nio.file.*;
import java.util.*;
import javax.imageio.ImageIO;
public class GlyphOracle {
  static Object get(Object o,String n)throws Exception {return o.getClass().getMethod(n).invoke(o);}
  @SuppressWarnings({"rawtypes","unchecked"})
  public static void main(String[]args)throws Exception {
    BufferedImage image=ImageIO.read(Path.of(args[0]).toFile());
    Class<?> bp=Class.forName("ij.process.ByteProcessor"), oc=Class.forName("org.audiveris.omr.run.Orientation"), rc=Class.forName("org.audiveris.omr.run.RunTable");
    Object pixels=bp.getConstructor(BufferedImage.class).newInstance(image);
    Object orientation=Enum.valueOf((Class)oc,args[1]);
    Object factory=Class.forName("org.audiveris.omr.run.RunTableFactory").getConstructor(oc).newInstance(orientation);
    Object table=factory.getClass().getMethod("createTable",bp).invoke(factory,pixels);
    List<?> glyphs=(List<?>)Class.forName("org.audiveris.omr.glyph.GlyphFactory").getMethod("buildGlyphs",rc,Point.class).invoke(null,table,new Point(Integer.parseInt(args[2]),Integer.parseInt(args[3])));
    StringJoiner result=new StringJoiner(",","[","]");
    for(Object g:glyphs) {
      Object rt=get(g,"getRunTable");StringJoiner runs=new StringJoiner(",","[","]");int size=(Integer)get(rt,"getSize");
      for(int p=0;p<size;p++) {
        Iterator<?> it=(Iterator<?>)rc.getMethod("iterator",int.class).invoke(rt,p);
        while(it.hasNext()) {Object r=it.next();runs.add("["+p+","+get(r,"getStart")+","+get(r,"getLength")+"]");}
      }
      result.add("{\"left\":"+get(g,"getLeft")+",\"top\":"+get(g,"getTop")+",\"width\":"+get(g,"getWidth")+",\"height\":"+get(g,"getHeight")+",\"weight\":"+get(g,"getWeight")+",\"runs\":"+runs+"}");
    }
    Files.writeString(Path.of(args[4]),result.toString()+"\n");
    System.exit(0);
  }
}
