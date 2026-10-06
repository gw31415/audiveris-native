// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class ChordStemDirectionProbe extends FoundationOverlapProbe {
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> pages=new ArrayList<>();
  for(int i=1;i<args.length;i++) {
   Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[i])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<Object> chords=new ArrayList<>();
   for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object stack:(List<?>)call(sys,"getStacks"))for(Object ch:(Collection<?>)call(call(stack,"getFirstMeasure"),"getHeadChords")) {
    chords.add(obj("system_id",call(sys,"getId"),"measure_number",call(stack,"getIdValue"),"bounds",rect(call(ch,"getBounds")),"stem_direction_code",call(ch,"getStemDir")));
   }
   pages.add(obj("input",Path.of(args[i]).getFileName().toString(),"chords",chords));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[0]).toFile(),obj("scope","actual original saved HeadChord.getStemDir, same successful images; not automatic general slot matching","pages",pages));System.exit(0);
 }
}
