// SPDX-License-Identifier: AGPL-3.0-or-later
// Explicit bridges on independently rendered accidental rasters. These inputs
// test original hard-slice fallback; they contain no native recognition output.
import java.awt.*;
import java.awt.image.BufferedImage;
import java.nio.file.*;
import java.util.*;
import javax.imageio.ImageIO;

public class MakeFallbackHeaderFixtures {
    static String sha(Path path) throws Exception {
        return HexFormat.of().formatHex(java.security.MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(path)));
    }
    public static void main(String[] args) throws Exception {
        Path input=Path.of(args[0]),output=Path.of(args[1]);Files.createDirectories(output);
        String[] names={"flats-bridge-120","flats-bridge-125","flats-bridge-130","flats-thick-bridge-125","sharps-bridge-125","sharps-bridge-130"};
        int[][] rects={{175,120,23,1},{175,125,23,1},{175,130,23,1},{175,125,23,2},{178,125,25,1},{178,130,25,1}};
        java.util.List<String> records=new ArrayList<>();
        for(int i=0;i<names.length;i++){
            String filename=names[i].startsWith("flats")?"rendered-header-four-flats.png":"rendered-header-three-sharps.png";
            Path source=input.resolve(filename);BufferedImage image=ImageIO.read(source.toFile());Graphics2D g=image.createGraphics();g.setColor(Color.BLACK);
            int[] r=rects[i];g.fillRect(r[0],r[1],r[2],r[3]);g.dispose();Path target=output.resolve("fallback-header-"+names[i]+".png");ImageIO.write(image,"PNG",target.toFile());
            records.add("{\"name\":\""+names[i]+"\",\"source_filename\":\""+filename+"\",\"source_sha256\":\""+sha(source)
                +"\",\"perturbation\":{\"fill\":\"black\",\"rectangle\":"+Arrays.toString(r)+"},\"output_filename\":\""+target.getFileName()+"\",\"output_sha256\":\""+sha(target)+"\"}");
        }
        Files.writeString(output.resolve("perturbations.json"),"{\"synthetic_input\":true,\"native_outputs_used\":false,\"variants\":["+String.join(",",records)+"]}\n");
    }
}
