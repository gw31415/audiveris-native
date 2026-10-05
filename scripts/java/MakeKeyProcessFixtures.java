// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent rendered inputs plus explicit bridges/trailing-ink perturbations.
import java.awt.*;import java.awt.image.BufferedImage;import java.nio.file.*;import java.util.*;import javax.imageio.ImageIO;
public class MakeKeyProcessFixtures {
    static String sha(Path p)throws Exception{return HexFormat.of().formatHex(java.security.MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(p)));}
    public static void main(String[] args)throws Exception{
        Path input=Path.of(args[0]),out=Path.of(args[1]);Files.createDirectories(out);
        String[] names={"flats-upper-bridge-123","flats-upper-bridge-122","one-flat-trailing-block","one-sharp-trailing-block"};
        String[] source={"rendered-header-four-flats.png","rendered-header-four-flats.png","rendered-header-one-flat.png","rendered-header-one-sharp.png"};
        int[][] rects={{175,123,23,2},{175,122,23,3},{191,146,20,15},{192,110,20,16}};
        java.util.List<String> records=new ArrayList<>();
        for(int i=0;i<names.length;i++){
            Path parent=input.resolve(source[i]);BufferedImage image=ImageIO.read(parent.toFile());Graphics2D g=image.createGraphics();g.setColor(Color.BLACK);int[] r=rects[i];g.fillRect(r[0],r[1],r[2],r[3]);g.dispose();Path target=out.resolve("key-process-"+names[i]+".png");ImageIO.write(image,"PNG",target.toFile());
            records.add("{\"name\":\""+names[i]+"\",\"source_filename\":\""+source[i]+"\",\"source_sha256\":\""+sha(parent)+"\",\"perturbation\":{\"fill\":\"black\",\"rectangle\":"+Arrays.toString(r)+"},\"output_filename\":\""+target.getFileName()+"\",\"output_sha256\":\""+sha(target)+"\"}");
        }
        Files.writeString(out.resolve("perturbations.json"),"{\"synthetic_input\":true,\"native_outputs_used\":false,\"variants\":["+String.join(",",records)+"]}\n");
    }
}
