// Development-only independent score input. No native recognition outputs.
import java.awt.*;
import java.awt.image.BufferedImage;
import java.nio.file.*;
public class LimitedScoreRenderer {
 static void staff(BufferedImage p, Graphics2D g,int top,boolean meter)throws Exception {
  for(int j=0;j<5;j++)g.fillRect(80,top+j*11,1120,2);
  for(int x:new int[]{80,710,1198})g.fillRect(x,top,2,46);
  HeadActualPageRenderer.stamp(p,HeadActualPageRenderer.symbol("Bravura","G_CLEF",51),100,top+22);
  if(meter)for(int y:new int[]{top+11,top+33})HeadActualPageRenderer.stamp(p,HeadActualPageRenderer.symbol("Bravura","TIME_FOUR",51),165,y);
 }
 static void event(BufferedImage p,Graphics2D g,int top,int x,int pitch,String shape)throws Exception {
  int cy=top+22+pitch*11/2;
  if(shape.equals("HALF_REST"))cy=top+20;
  BufferedImage glyph=HeadActualPageRenderer.symbol("Bravura",shape,51);
  HeadActualPageRenderer.stamp(p,glyph,x,cy);
  if(shape.startsWith("NOTEHEAD"))g.fillRect(x+glyph.getWidth()-(shape.equals("NOTEHEAD_VOID")?6:2),cy-36,2,37);
 }
 public static void main(String[] args)throws Exception {
  Path out=Path.of(args[0]);Files.createDirectories(out);
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"populateAllSymbols");
  for(String name:new String[]{"quarter-rest","mixed-four-measures","half-two-measures","quarter-half-break","central-rest-ambiguous","no-meter"}){
   int height=(name.equals("mixed-four-measures")||name.equals("quarter-half-break"))?760:420;
   BufferedImage p=new BufferedImage(1280,height,BufferedImage.TYPE_BYTE_GRAY);
   Graphics2D g=p.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,1280,height);g.setColor(Color.BLACK);
   staff(p,g,220,!name.equals("no-meter"));
   if(name.equals("central-rest-ambiguous")){
    int[] pitch={0,2,4,2};for(int i=0;i<4;i++)event(p,g,220,270+120*i,pitch[i],"NOTEHEAD_BLACK");
    event(p,g,220,750,0,"NOTEHEAD_VOID");event(p,g,220,990,0,"QUARTER_REST");event(p,g,220,1110,-2,"NOTEHEAD_BLACK");
   }else if(name.equals("quarter-half-break")){
    int[] pitch={0,2,4,2};
    for(int i=0;i<4;i++)event(p,g,220,270+120*i,pitch[i],i==1?"QUARTER_REST":"NOTEHEAD_BLACK");
    event(p,g,220,750,0,"NOTEHEAD_VOID");event(p,g,220,990,-2,"NOTEHEAD_VOID");
    staff(p,g,540,false);
    for(int i=0;i<4;i++)event(p,g,540,270+120*i,pitch[i],i==1?"QUARTER_REST":"NOTEHEAD_BLACK");
    event(p,g,540,750,-2,"NOTEHEAD_VOID");event(p,g,540,990,0,"NOTEHEAD_VOID");
   }else if(name.equals("half-two-measures")){
    for(int x:new int[]{270,510,750,990})event(p,g,220,x,(x==510||x==990)?-2:0,"NOTEHEAD_VOID");
   }else if(!name.equals("mixed-four-measures")){
    int[] pitch={0,2,4,2,0,-2,-4,-2};
    for(int i=0;i<8;i++)event(p,g,220,270+120*i,pitch[i],name.equals("quarter-rest")&&(i==1||i==5)?"QUARTER_REST":"NOTEHEAD_BLACK");
   }else{
    int[] pitch={0,2,4,2};for(int i=0;i<4;i++)event(p,g,220,270+120*i,pitch[i],"NOTEHEAD_BLACK");
    event(p,g,220,750,0,"NOTEHEAD_VOID");event(p,g,220,990,0,"QUARTER_REST");event(p,g,220,1110,-2,"NOTEHEAD_BLACK");
    staff(p,g,540,false);
    event(p,g,540,270,0,"HALF_REST");event(p,g,540,510,0,"NOTEHEAD_BLACK");event(p,g,540,630,-2,"NOTEHEAD_BLACK");
    event(p,g,540,750,-2,"NOTEHEAD_VOID");event(p,g,540,990,0,"NOTEHEAD_VOID");
   }
   g.dispose();javax.imageio.ImageIO.write(p,"PNG",out.resolve(name+".png").toFile());
  }
 }
}
