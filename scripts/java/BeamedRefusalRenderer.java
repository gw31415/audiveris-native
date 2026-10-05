import java.awt.*;import java.awt.image.BufferedImage;import java.nio.file.*;
public class BeamedRefusalRenderer {
 public static void main(String[] args)throws Exception{
  Path out=Path.of(args[0]);Files.createDirectories(out);
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  for(int size:new int[]{0}){
   BufferedImage p=new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,1280,420);g.setColor(Color.BLACK);LimitedScoreRenderer.staff(p,g,220,true);
   LimitedScoreRenderer.event(p,g,220,270,0,"NOTEHEAD_BLACK");LimitedScoreRenderer.event(p,g,220,390,0,"NOTEHEAD_BLACK");g.fillRect(288,206,122,6);
   LimitedScoreRenderer.event(p,g,220,510,0,"NOTEHEAD_BLACK");LimitedScoreRenderer.event(p,g,220,630,-2,"NOTEHEAD_BLACK");
   LimitedScoreRenderer.event(p,g,220,750,-2,"NOTEHEAD_VOID");LimitedScoreRenderer.event(p,g,220,990,0,"NOTEHEAD_VOID");g.dispose();javax.imageio.ImageIO.write(p,"PNG",out.resolve("beamed-notes.png").toFile());
  }
 }
}
