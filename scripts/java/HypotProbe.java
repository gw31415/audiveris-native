// Development-only original Math.hypot vectors, loaded from hexadecimal IEEE bits.
import java.nio.file.*;
import java.util.*;
public class HypotProbe {
 public static void main(String[] args) throws Exception {
  List<String> output = new ArrayList<>();
  for (String line : Files.readAllLines(Path.of(args[0]))) {
   String[] pair = line.trim().split(" ");
   double x = Double.longBitsToDouble(Long.parseUnsignedLong(pair[0],16));
   double y = Double.longBitsToDouble(Long.parseUnsignedLong(pair[1],16));
   output.add(pair[0]+" "+pair[1]+" "+Long.toUnsignedString(Double.doubleToLongBits(Math.hypot(x,y)),16));
  }
  Files.write(Path.of(args[1]),output);
 }
}
