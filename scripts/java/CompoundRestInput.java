// SPDX-License-Identifier: AGPL-3.0-or-later
// One bounded fixture from the already-saved successful quarter-rest image.
// A single one-pixel horizontal ink interruption; no position search.
import java.awt.*;import java.awt.image.*;import java.nio.file.*;import javax.imageio.ImageIO;
public class CompoundRestInput {
 public static void main(String[] args)throws Exception {
  BufferedImage p=ImageIO.read(Path.of(args[0]).toFile());Graphics2D g=p.createGraphics();
  g.setColor(Color.WHITE);g.fillRect(390,251,18,1);g.dispose();ImageIO.write(p,"PNG",Path.of(args[1]).toFile());
 }
}
