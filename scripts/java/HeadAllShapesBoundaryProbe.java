// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;
public class HeadAllShapesBoundaryProbe extends HeadRangeCatalogProbe {
 public HeadAllShapesBoundaryProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 void controls(Path output)throws Exception {
  List<String> rows=new ArrayList<>();
  for(String shape:new String[]{"BREVE_SMALL","BREVE_CROSS","BREVE_DIAMOND","BREVE_TRIANGLE_DOWN","BREVE_CIRCLE_X","WHOLE_NOTE_SMALL","WHOLE_NOTE_CROSS","WHOLE_NOTE_DIAMOND","WHOLE_NOTE_TRIANGLE_DOWN","WHOLE_NOTE_CIRCLE_X","NOTEHEAD_VOID_SMALL","NOTEHEAD_CROSS_VOID","NOTEHEAD_DIAMOND_VOID","NOTEHEAD_TRIANGLE_DOWN_VOID","NOTEHEAD_CIRCLE_X_VOID"})for(double score:new double[]{0.,Math.nextDown(.1875),.1875,Math.nextUp(.1875),Math.nextUp(Math.nextUp(.1875)),.4,Math.nextDown(.45),.45,Math.nextUp(.45),1.})rows.add("{\"shape\":\""+shape+"\",\"score_bits\":"+Double.doubleToRawLongBits(score)+",\"eligibility\":"+decision(shape,score,40,45)+"}");
  Files.writeString(output.resolveSibling("all-head-boundaries.json"),"{\"native_outputs_used\":false,\"boundaries\":["+String.join(",",rows)+"]}\n");
 }
}
