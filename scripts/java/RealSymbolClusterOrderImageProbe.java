// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealSymbolClusterOrderImageProbe extends RealSymbolCandidatesImageProbe {
 public RealSymbolClusterOrderImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);}
 public void process(){super.process();try{
  Path out=Path.of(System.getProperty("native.probe.output"));String s=Files.readString(out).trim();
  int hash=c("run.Orientation").getField("VERTICAL").get(null).hashCode();
  Files.writeString(out,s.substring(0,s.length()-1)+",\"vertical_orientation_identity_hash\":"+hash+"}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
