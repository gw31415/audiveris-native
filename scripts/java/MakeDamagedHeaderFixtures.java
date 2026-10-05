// SPDX-License-Identifier: AGPL-3.0-or-later
// Explicit pixel perturbations of independently rendered Bravura test inputs.
// No native recognition output or oracle answer is used to draw these rasters.
import java.awt.*;
import java.awt.image.BufferedImage;
import java.nio.file.*;
import java.util.*;
import javax.imageio.ImageIO;

public class MakeDamagedHeaderFixtures {
    static String sha(Path path) throws Exception {
        return HexFormat.of().formatHex(java.security.MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(path)));
    }
    public static void main(String[] args) throws Exception {
        Path input = Path.of(args[0]), output = Path.of(args[1]);
        Files.createDirectories(output);
        String[] names = {"flat-cut-128", "flat-cut-136", "flat-cut-143", "flat-cut-147",
            "sharp-cut-110", "sharp-cut-120", "sharp-cut-130", "flat-speck-125", "flat-speck-132", "sharp-speck-98"};
        int[][] rectangles = {{172,128,20,1},{172,136,20,1},{172,143,20,1},{172,147,20,1},
            {172,110,22,1},{172,120,22,1},{172,130,22,1},{182,125,2,2},{183,132,3,3},{181,98,2,2}};
        java.util.List<String> records = new ArrayList<>();
        for (int i=0;i<names.length;i++) {
            String base = names[i].startsWith("flat") ? "rendered-header-one-flat.png" : "rendered-header-one-sharp.png";
            Path source = input.resolve(base);
            BufferedImage image = ImageIO.read(source.toFile());
            Graphics2D g = image.createGraphics();
            boolean cut = names[i].contains("cut");
            g.setColor(cut ? Color.WHITE : Color.BLACK);
            int[] rect = rectangles[i];
            g.fillRect(rect[0],rect[1],rect[2],rect[3]);
            g.dispose();
            Path target = output.resolve("damaged-header-" + names[i] + ".png");
            ImageIO.write(image,"PNG",target.toFile());
            records.add("{\"name\":\"" + names[i] + "\",\"source_filename\":\"" + base + "\",\"source_sha256\":\"" + sha(source)
                + "\",\"perturbation\":{\"fill\":\"" + (cut ? "white" : "black") + "\",\"rectangle\":" + Arrays.toString(rect)
                + "},\"output_filename\":\"" + target.getFileName() + "\",\"output_sha256\":\"" + sha(target) + "\"}");
        }
        Files.writeString(output.resolve("perturbations.json"), "{\"synthetic_input\":true,\"native_outputs_used\":false,\"variants\":[" + String.join(",",records) + "]}\n");
    }
}
