// SPDX-License-Identifier: AGPL-3.0-or-later
// Read original owned intrinsic shortening; no declared music or clock edits.
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class ChordShorteningProbe extends RookieMappingProbe {
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<Object> rows=new ArrayList<>();
  for(Object system:(List<?>)call(sheet,"getSystems"))for(Object stack:(List<?>)call(system,"getStacks")){
   Object measure=call(stack,"getFirstMeasure");
   for(Object chord:(Collection<?>)call(measure,"getStandardChords"))rows.add(obj("measure_number",call(stack,"getIdValue"),"bounds",rect(call(chord,"getBounds")),"beams",((Collection<?>)call(chord,"getBeams")).size(),"flags",call(chord,"getFlagsNumber"),"beams_or_flags",call(chord,"getBeamsOrFlagsNumber"),"intrinsic",fraction(call(chord,"getDurationSansDotOrTuplet"))));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","read-only actual original recognized chords; intrinsic shortening getters, no duration/Voice/clock changes","chords",rows));System.exit(0);
 }
}
