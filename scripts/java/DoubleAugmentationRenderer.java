// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.*;
import java.awt.image.BufferedImage;
import java.nio.file.Path;
public class DoubleAugmentationRenderer {
 public static void main(String[] args) throws Exception {
  HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"), "populateAllSymbols");
  BufferedImage page = new BufferedImage(1280,420,BufferedImage.TYPE_BYTE_GRAY);
  Graphics2D g=page.createGraphics(); g.setColor(Color.WHITE);g.fillRect(0,0,1280,420);g.setColor(Color.BLACK);
  LimitedScoreRenderer.staff(page,g,220,true);
  LimitedScoreRenderer.event(page,g,220,270,0,"NOTEHEAD_BLACK");
  BufferedImage dot=HeadActualPageRenderer.symbol("Bravura","AUGMENTATION_DOT",51);
  HeadActualPageRenderer.stamp(page,dot,302,237);
  HeadActualPageRenderer.stamp(page,dot,316,237);
  LimitedScoreRenderer.event(page,g,220,390,-2,"NOTEHEAD_BLACK");
  Object family=HeadActualPageRenderer.cls("ui.symbol.MusicFamily").getField("Bravura").get(null);
  Object font=HeadActualPageRenderer.call(HeadActualPageRenderer.cls("ui.symbol.MusicFont"),"getMusicFont",family,51);
  Object shape=HeadActualPageRenderer.cls("glyph.Shape").getField("FLAG_2").get(null);
  java.awt.font.TextLayout flag=(java.awt.font.TextLayout)HeadActualPageRenderer.call(font,"layoutShapeByCode",shape);
  g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_ON);flag.draw(g,408,195);
  g.setRenderingHint(RenderingHints.KEY_ANTIALIASING,RenderingHints.VALUE_ANTIALIAS_OFF);
  LimitedScoreRenderer.event(page,g,220,510,-4,"NOTEHEAD_BLACK");
  LimitedScoreRenderer.event(page,g,220,630,-2,"NOTEHEAD_BLACK");
  int[] pitches={-4,-2,0,-2};for(int i=0;i<4;i++)LimitedScoreRenderer.event(page,g,220,750+120*i,pitches[i],"NOTEHEAD_BLACK");
  g.dispose();javax.imageio.ImageIO.write(page,"PNG",Path.of(args[0]).toFile());
 }
}
