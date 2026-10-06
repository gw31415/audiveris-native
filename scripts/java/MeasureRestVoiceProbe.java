// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class MeasureRestVoiceProbe extends FoundationOverlapProbe {
 static Object rational(Object value){return value==null?null:value.toString();}
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0);
  List<Object> rows=new ArrayList<>();
  for(Object stack:(List<?>)call(sys,"getStacks")) {
   Object measure=call(stack,"getFirstMeasure");List<Object> voices=new ArrayList<>();
   for(Object voice:(List<?>)call(measure,"getVoices")) {
    call(voice,"checkDuration");Object whole=call(voice,"getWholeChord"),last=call(voice,"getLastChord");
    voices.add(obj("id",call(voice,"getId"),"is_measure_rest",call(voice,"isMeasureRest"),"chord_count",((List<?>)call(voice,"getChords")).size(),"measure_rest_bounds",whole==null?null:rect(call(whole,"getBounds")),"chord_duration",whole==null?null:rational(call(whole,"getDuration")),"duration",rational(call(voice,"getDuration")),"end_time",last==null?null:rational(call(last,"getEndTime")),"termination",rational(call(voice,"getTermination"))));
   }
   rows.add(obj("number",call(stack,"getIdValue"),"slot_count",((List<?>)call(stack,"getSlots")).size(),"voices",voices,"expected_duration",rational(call(stack,"getExpectedDuration")),"actual_duration",rational(call(stack,"getActualDuration")),"abnormal",call(measure,"isAbnormal")));
  }
  Object switches=call(call(sheet,"getStub"),"getProcessingSwitches");
  Object partial=c("sheet.ProcessingSwitch").getField("partialWholeRests").get(null);List<Object> shapeStates=new ArrayList<>();
  for(boolean value:new boolean[]{false,true}) {call(call(switches,"getParam",partial),"setSpecific",value);List<Object> shapes=new ArrayList<>();for(String name:new String[]{"WHOLE_REST","BREVE_REST","LONG_REST","QUARTER_REST"}) shapes.add(obj("shape",name,"measure_rest",call(sys,"isMeasureRestShape",c("glyph.Shape").getField(name).get(null))));shapeStates.add(obj("partialWholeRests",value,"shapes",shapes));}
  call(call(switches,"getParam",partial),"setSpecific",false);
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original image full pipeline -> normal/whole-measure-rest Voice, null intrinsic duration and no-slot expected/actual duration; default partialWholeRests=false","partialWholeRests",call(switches,"getValue",c("sheet.ProcessingSwitch").getField("partialWholeRests").get(null)),"measure_rest_shapes",shapeStates,"measures",rows));System.exit(0);
 }
}
