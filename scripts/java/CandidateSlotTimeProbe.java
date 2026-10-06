// SPDX-License-Identifier: AGPL-3.0-or-later
// Actual original transient narrow slot clocks before/after SlotMapper.
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class CandidateSlotTimeProbe extends RookieMappingProbe {
 static Object clocks(Object slot)throws Exception {
  List<Object> result=new ArrayList<>();for(Object narrow:(List<?>)call(slot,"getMembers"))result.add(obj("id",call(narrow,"getId"),"time_offset",fraction(call(narrow,"getTimeOffset"))));return result;
 }
 static Object inspectTimes(Object measure)throws Exception {
  call(measure,"resetRhythm");Object rhythm=make("sheet.rhythm.MeasureRhythm",measure),retriever=make("sheet.rhythm.SlotsRetriever",measure,false);put(rhythm,"narrowSlotsRetriever",retriever);
  List<?> narrow=(List<?>)call(retriever,"buildSlots"),wide=(List<?>)call(make("sheet.rhythm.SlotsRetriever",measure,true),"buildSlots"),slots=(List<?>)invoke(rhythm,"buildCompoundSlots",narrow,wide);((List)field(rhythm,"slots")).addAll(slots);invoke(rhythm,"processStartingChords");
  Class<?> mapperClass=c("sheet.rhythm.MeasureRhythm$SlotMapper");var ctor=mapperClass.getDeclaredConstructor(c("sheet.rhythm.MeasureRhythm"),c("sheet.rhythm.Slot$CompoundSlot"));ctor.setAccessible(true);List<Object> steps=new ArrayList<>();
  for(Object slot:slots){Object before=clocks(slot);boolean ok=(Boolean)invoke(ctor.newInstance(rhythm,slot),"mapChords");steps.add(obj("slot_id",call(slot,"getId"),"before",before,"after",clocks(slot),"ok",ok));invoke(rhythm,"purgeExtinctVoices",slot);}
  return obj("number",call(call(measure,"getStack"),"getIdValue"),"steps",steps);
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> pages=new ArrayList<>();
  for(int i=1;i<args.length;i++){Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[i])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<Object> measures=new ArrayList<>();for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object stack:(List<?>)call(sys,"getStacks"))measures.add(inspectTimes(call(stack,"getFirstMeasure")));pages.add(obj("input",Path.of(args[i]).getFileName().toString(),"measures",measures));}
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[0]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","two saved successful original image states; transient narrow slot getter before/after mapChords; not native general time conflict/retry/polyphony","pages",pages));System.exit(0);
 }
}
