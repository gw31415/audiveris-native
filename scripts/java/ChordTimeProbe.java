// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class ChordTimeProbe extends FoundationOverlapProbe {
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> controls=new ArrayList<>();
  for(String mode:new String[]{"group","sequence","tie","cross-measure","null"}) {
   Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
   Object group=((Collection<?>)call(sig,"vertexSet")).stream().filter(n->n.getClass().getSimpleName().equals("BeamGroupInter")).findFirst().get(),voice=call(call(group,"getFirstChord"),"getVoice"),seed=call(group,"getFirstChord"),last=call(group,"getLastChord");
   List<?> chords=(List<?>)call(voice,"getChords");
   for(Object chord:chords) call(chord,"setTimeOffset",(Object)null);
   Object nextMeasure=((List<?>)call(sys,"getStacks")).get(1),other=call(((List<?>)call(call(nextMeasure,"getFirstMeasure"),"getVoices")).get(0),"getFirstChord");
   if(mode.equals("sequence")) {
    call(sig,"addEdge",last,chords.get(4),make("sig.relation.NextInVoiceRelation"));
    call(sig,"addEdge",chords.get(4),chords.get(5),make("sig.relation.NextInVoiceRelation"));
   } else if(mode.equals("tie")) {
    Object slur=make("sig.inter.SlurInter",true,.8);call(sig,"addVertex",slur);
    call(sig,"addEdge",slur,((List<?>)call(last,"getMembers")).get(0),make("sig.relation.SlurHeadRelation",c("util.HorizontalSide").getField("LEFT").get(null)));
    call(sig,"addEdge",slur,((List<?>)call(chords.get(4),"getMembers")).get(0),make("sig.relation.SlurHeadRelation",c("util.HorizontalSide").getField("RIGHT").get(null)));call(slur,"setTie",true);
    call(sig,"addEdge",chords.get(4),chords.get(5),make("sig.relation.NextInVoiceRelation"));
   } else if(mode.equals("cross-measure")) call(sig,"addEdge",last,other,make("sig.relation.NextInVoiceRelation"));
   call(seed,"setAndPushTime",mode.equals("null")?null:c("math.Rational").getField("ZERO").get(null));
   List<Object> times=new ArrayList<>();for(Object chord:chords)times.add(obj("bounds",rect(call(chord,"getBounds")),"time_offset",call(chord,"getTimeOffset")==null?null:call(chord,"getTimeOffset").toString()));
   controls.add(obj("mode",mode,"chords",times,"other_measure_offset",call(other,"getTimeOffset").toString()));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original actual beam group clocks, declared NextInVoice/classified tie/cross-measure/null controls; not automatic relation acquisition or tie MusicXML","controls",controls));System.exit(0);
 }
}
