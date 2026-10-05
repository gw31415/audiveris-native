// SPDX-License-Identifier: AGPL-3.0-or-later
// Declared Java2D PageCleaner raster controls, no native outputs.
import java.awt.*;import java.awt.geom.*;import java.awt.image.*;import java.nio.file.*;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class BeamRectRasterProbe {
 public static void main(String[] args)throws Exception{
  java.util.List<Object> out=new ArrayList<>();
  for(double[] r:new double[][]{{10,20,30,26},{10,19.95,30,26.05},{10,20.05,30,25.95},{10.1,20.05,30.1,25.95},{10,20.5,30,25.5},{10,19.7499,30,26.7499},{10,19.75,30,26.75},{10,19.7501,30,26.7501}}){
   BufferedImage im=new BufferedImage(50,50,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=im.createGraphics();g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_OFF);g.setColor(Color.WHITE);g.setStroke(new BasicStroke(1.1f,BasicStroke.CAP_SQUARE,BasicStroke.JOIN_MITER));Area a=new Area(new Rectangle2D.Double(r[0],r[1],r[2]-r[0],r[3]-r[1]));g.fill(a);g.draw(a);g.dispose();java.util.List<Object> rows=new ArrayList<>();for(int y=0;y<50;y++){int min=50,max=-1;for(int x=0;x<50;x++)if(im.getRaster().getSample(x,y,0)==255){min=Math.min(min,x);max=Math.max(max,x);}if(max>=0)rows.add(new int[]{y,min,max+1});}out.add(Map.of("area_bounds",r,"white_rows",rows));
  }
  new ObjectMapper().writeValue(Path.of(args[0]).toFile(),Map.of("scope","eight declared rectangular beam-area Java2D AA-off normalized one-pixel contour controls; not image coordinate search","cases",out));
 }
}
