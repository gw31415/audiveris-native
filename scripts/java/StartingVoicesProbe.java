// SPDX-License-Identifier: AGPL-3.0-or-later
// Actual original starting Voice constructor -> setVoice/group -> pushTime(0),
// after geometric compound retrieval, before later rookie/active mapping.
import java.nio.file.Path;import java.util.*;import java.lang.reflect.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class StartingVoicesProbe extends FoundationOverlapProbe {
 static Object fraction(Object value){return value==null?null:value.toString();}
 static Object inspect(Object measure)throws Exception {
  call(measure,"resetRhythm");List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getStandardChords"));
  Object rhythm=make("sheet.rhythm.MeasureRhythm",measure);List<?> narrow=(List<?>)call(make("sheet.rhythm.SlotsRetriever",measure,false),"buildSlots"),wide=(List<?>)call(make("sheet.rhythm.SlotsRetriever",measure,true),"buildSlots");
  Method compound=c("sheet.rhythm.MeasureRhythm").getDeclaredMethod("buildCompoundSlots",List.class,List.class);compound.setAccessible(true);List<?> built=(List<?>)compound.invoke(rhythm,narrow,wide);
  Field slots=c("sheet.rhythm.MeasureRhythm").getDeclaredField("slots");slots.setAccessible(true);((List)slots.get(rhythm)).addAll(built);
  Method starting=c("sheet.rhythm.MeasureRhythm").getDeclaredMethod("processStartingChords");starting.setAccessible(true);starting.invoke(rhythm);
  List<Object> voices=new ArrayList<>(),states=new ArrayList<>();
  for(Object v:(List<?>)call(measure,"getVoices")){List<Object> ids=new ArrayList<>();for(Object ch:(List<?>)call(v,"getChords"))ids.add(chords.indexOf(ch));Object rest=call(v,"getWholeChord");voices.add(obj("id",call(v,"getId"),"is_measure_rest",call(v,"isMeasureRest"),"chord_ids",ids,"measure_rest_chord_id",rest==null?null:chords.indexOf(rest)));}
  for(Object ch:chords){Object voice=call(ch,"getVoice");states.add(obj("id",chords.indexOf(ch),"bounds",rect(call(ch,"getBounds")),"duration",fraction(call(ch,"getDuration")),"time_offset",fraction(call(ch,"getTimeOffset")),"voice",voice==null?null:call(voice,"getId")));}
  return obj("number",call(call(measure,"getStack"),"getIdValue"),"voices",voices,"chords",states);
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> pages=new ArrayList<>();
  for(int i=1;i<args.length;i++){Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[i])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<Object> measures=new ArrayList<>();for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object stack:(List<?>)call(sys,"getStacks"))measures.add(inspect(call(stack,"getFirstMeasure")));pages.add(obj("input",Path.of(args[i]).getFileName().toString(),"measures",measures));}
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[0]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","saved successful same-image states at processStartingChords before general rookie mapping; real beam Voice/time propagation and dedicated default whole-rest Voice, not automatic general polyphony","pages",pages));System.exit(0);
 }
}
