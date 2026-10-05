import java.awt.*;
import java.awt.image.BufferedImage;
import java.nio.file.*;
public class MinimalMelodyRenderer {
 public static void main(String[] args) throws Exception {
  Path out=Path.of(args[0]);Files.createDirectories(out);
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  BufferedImage page=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);
  Graphics2D g=page.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,1280,420);g.setColor(Color.BLACK);
  int top=220,il=11;
  for(int j=0;j<5;j++)g.fillRect(80,top+j*il,1120,2);
  for(int x:new int[]{80,710,1198})g.fillRect(x,top,2,46);
  HeadActualPageRenderer.stamp(page,HeadActualPageRenderer.symbol("Bravura","G_CLEF",51),100,top+22);
  for(int y:new int[]{top+11,top+33})HeadActualPageRenderer.stamp(page,HeadActualPageRenderer.symbol("Bravura","TIME_FOUR",51),165,y);
  int[] pitch={0,2,4,2,0,-2,-4,-2};
  for(int i=0;i<8;i++){
   int x=270+120*i,cy=top+22+pitch[i]*il/2;
   BufferedImage head=HeadActualPageRenderer.symbol("Bravura","NOTEHEAD_BLACK",51);
   HeadActualPageRenderer.stamp(page,head,x,cy);g.fillRect(x+head.getWidth()-2,cy-36,2,37);
  }
  g.dispose();javax.imageio.ImageIO.write(page,"PNG",out.resolve("quarter-melody.png").toFile());
 }
}
