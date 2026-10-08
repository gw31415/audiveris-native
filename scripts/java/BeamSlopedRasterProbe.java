// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.*;
import java.awt.geom.*;
import java.awt.image.*;
import java.nio.file.*;
import java.util.ArrayList;
public class BeamSlopedRasterProbe {
 public static void main(String[] args)throws Exception {
  java.util.List<String> out=new ArrayList<>();
  for(double[] p:new double[][]{{288,209,470,210,6.1},{288,209.7,350,209.8,6},{10,20,50,22.4,6},{10,22.4,50,20,6},{10.2499,20.7499,50.2499,23.7499,6},{10.25,20.75,50.25,23.75,6},{10.2501,20.7501,50.2501,23.7501,6},{10,20,12,21,1.8}}){
   Path2D path=new Path2D.Double();path.moveTo(p[0],p[1]-p[4]/2);path.lineTo(p[2],p[3]-p[4]/2);path.lineTo(p[2],p[3]+p[4]/2);path.lineTo(p[0],p[1]+p[4]/2);path.closePath();Area area=new Area(path);
   BufferedImage im=new BufferedImage(512,512,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=im.createGraphics();g.setColor(Color.WHITE);g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_OFF);g.setStroke(new BasicStroke(1.1f,BasicStroke.CAP_SQUARE,BasicStroke.JOIN_MITER));g.fill(area);g.draw(area);g.dispose();java.util.List<String> rows=new ArrayList<>();
   for(int y=0;y<512;y++){int left=-1,right=-1;for(int x=0;x<512;x++)if(im.getRaster().getSample(x,y,0)!=0){if(left<0)left=x;right=x+1;}if(left>=0)rows.add("["+y+","+left+","+right+"]");}
   out.add("{\"kind\":\"BeamInter\",\"median\":["+p[0]+","+p[1]+","+p[2]+","+p[3]+"],\"height\":"+p[4]+",\"white_rows\":["+String.join(",",rows)+"]}");
  }
  Files.writeString(Path.of(args[0]),"{\"scope\":\"eight declared original PageCleaner parallelogram/Java2D AA-off margin1.1 fill plus closed miter contour controls; separate from real image observations\",\"native_outputs_used\":false,\"cases\":["+String.join(",",out)+"]}\n");
 }
}
