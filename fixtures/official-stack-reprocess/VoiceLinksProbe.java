// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class VoiceLinksProbe extends VoiceRefineProbe {
 static void refine(Object sys)throws Exception {c("sheet.rhythm.Voices").getMethod("refineSystem",c("sheet.SystemInfo")).invoke(null,sys);}
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),page=call(sys,"getPage"),rhythm=make("sheet.rhythm.PageRhythm",page);List<?> stacks=(List<?>)call(sys,"getStacks");for(Object stack:stacks)call(rhythm,"reprocessStack",stack);
  Object previous=call(stacks.get(0),"getFirstMeasure"),current=call(stacks.get(1),"getFirstMeasure");List<?> pv=(List<?>)call(previous,"getVoices"),cv=(List<?>)call(current,"getVoices");Object before=voices(current),from=call(pv.get(1),"getFirstChord"),to=call(cv.get(0),"getFirstChord");call(sig,"addEdge",from,to,make("sig.relation.NextInVoiceRelation"));refine(sys);Object afterNext=voices(current);call(to,"setPreferredVoiceId",1);refine(sys);Object afterPreferred=voices(current);
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original rhythm-created owned parallel Voices; explicit saved-control NextInVoice across actual adjacent Measures, then preferred ID1; no automatic link discovery/image correction claim","source_first_chord",rect(call(from,"getBounds")),"source_voice_id",call(call(from,"getVoice"),"getId"),"target_first_chord",rect(call(to,"getBounds")),"before",before,"after_next",afterNext,"after_preferred",afterPreferred));System.exit(0);
 }
}
