// SPDX-License-Identifier: AGPL-3.0-or-later
// Original ImageJ buffer/image ownership control used by StemScaler.getBuffer.
import java.awt.*;import java.awt.image.*;import java.nio.file.*;import java.util.*;
public class StemBufferControlProbe {
 public static void main(String[]args)throws Exception{
  Object buffer=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class).newInstance(5,4);byte[] p=(byte[])GridProbe.call(buffer,"getPixels");for(int i=0;i<p.length;i++)p[i]=(byte)(i%3==0?0:255);byte[] before=p.clone();BufferedImage img=(BufferedImage)GridProbe.call(buffer,"getBufferedImage");Graphics2D g=img.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,5,4);g.dispose();byte[] after=((byte[])GridProbe.call(buffer,"getPixels")).clone();java.util.List<Integer> image=new ArrayList<>();for(int y=0;y<4;y++)for(int x=0;x<5;x++)image.add(img.getRaster().getSample(x,y,0));
  Files.writeString(Path.of(args[0]),"{\"controlled_input\":true,\"method\":\"original ByteProcessor.getBufferedImage then Graphics2D white fill\",\"width\":5,\"height\":4,\"before_base64\":\""+Base64.getEncoder().encodeToString(before)+"\",\"after_buffer_base64\":\""+Base64.getEncoder().encodeToString(after)+"\",\"painted_image_samples\":"+image+",\"buffer_unchanged\":"+Arrays.equals(before,after)+"}\n");System.exit(0);
 }
}
