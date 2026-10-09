// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealOrnamentStagesImageProbe extends RealOrnamentFactoryImageProbe {
 private final Object actualBook;
 public RealOrnamentStagesImageProbe(Book b,SortedSet<Integer> ids){super(b,ids);actualBook=b;}
 public void process(){try{Object stub=((java.util.List<?>)call(actualBook,"getStubs")).get(0);java.util.List<Object> frames=new ArrayList<>();for(String stage:java.util.List.of("SYMBOLS","LINKS","RHYTHMS")){call(stub,"reachStep",c("step.OmrStep").getField(stage).get(null),false);Object sheet=call(stub,"getSheet");for(Object sys:(java.util.List<?>)call(sheet,"getSystems"))frames.add(frameOrnaments(sys,"after_ordinary_"+stage));}Files.writeString(Path.of(System.getProperty("native.probe.output")),json(map("scope","Separate fresh unchanged AnLaura PNG through ordinary original SYMBOLS, LINKS and RHYTHMS via same stub reachStep, without repeated Symbols or manual lifecycle calls. This is independent of the actual factory observation and does not infer factory NULL from final absence.","frames",frames,"native_outputs_used",false))+"\n");}catch(Exception e){throw new RuntimeException(e);}}
}
