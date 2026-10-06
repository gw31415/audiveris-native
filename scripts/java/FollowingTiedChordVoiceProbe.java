// SPDX-License-Identifier: AGPL-3.0-or-later
// Existing original page, declared already-classified tie relations and reload fields.
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class FollowingTiedChordVoiceProbe extends FoundationOverlapProbe {
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> controls=new ArrayList<>();
  for(String mode:new String[]{"same-measure","other-measure","not-tie","source-on-right"}) {
   Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
   Object group=((Collection<?>)call(sig,"vertexSet")).stream().filter(n->n.getClass().getSimpleName().equals("BeamGroupInter")).findFirst().get(),voice=call(call(group,"getFirstChord"),"getVoice");List<?> chords=(List<?>)call(voice,"getChords");
   Object seed=chords.get(4),target=mode.equals("other-measure")?((Collection<?>)call(call(((List<?>)call(sys,"getStacks")).get(1),"getFirstMeasure"),"getHeadChords")).iterator().next():chords.get(5);
   Object sourceHead=((List<?>)call(seed,"getMembers")).get(0),targetHead=((List<?>)call(target,"getMembers")).get(0),slur=make("sig.inter.SlurInter",true,.8);call(sig,"addVertex",slur);
   Object left=c("util.HorizontalSide").getField("LEFT").get(null),right=c("util.HorizontalSide").getField("RIGHT").get(null);
   call(sig,"addEdge",slur,sourceHead,make("sig.relation.SlurHeadRelation",mode.equals("source-on-right")?right:left));call(sig,"addEdge",slur,targetHead,make("sig.relation.SlurHeadRelation",mode.equals("source-on-right")?left:right));call(slur,"setTie",!mode.equals("not-tie"));
   List<Object> following=new ArrayList<>();for(Object ch:(List<?>)call(seed,"getFollowingTiedChords"))following.add(obj("kind",ch.getClass().getSimpleName(),"bounds",rect(call(ch,"getBounds")),"measure_number",call(call(call(ch,"getMeasure"),"getStack"),"getIdValue")));
   call(seed,"justAssignVoice",(Object)null);call(target,"justAssignVoice",(Object)null);call(seed,"setVoice",voice);Object targetVoice=call(target,"getVoice");
   controls.add(obj("mode",mode,"is_tie",call(slur,"isTie"),"seed_bounds",rect(call(seed,"getBounds")),"target_bounds",rect(call(target,"getBounds")),"following",following,"target_voice",targetVoice==null?null:call(targetVoice,"getId")));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","four declared already-classified SlurInter/SlurHead side and same/other-measure controls; saved Voice fields reset via reload helper then setVoice; not automatic tie recognition,checkStaffTie or MusicXML tie export","controls",controls));System.exit(0);
 }
}
