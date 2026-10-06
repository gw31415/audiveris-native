// SPDX-License-Identifier: AGPL-3.0-or-later
// Controlled mutation of an existing original page: same/other measure BeamRest.
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class BeamGroupVoicePropagationProbe extends FoundationOverlapProbe {
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> controls=new ArrayList<>();
        for(boolean same:new boolean[]{true,false}) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
            Object sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),staff=call(sys,"getFirstStaff");
            Object group=((Collection<?>)call(sig,"vertexSet")).stream().filter(n->n.getClass().getSimpleName().equals("BeamGroupInter")).findFirst().get();
            Object beam=((List<?>)call(group,"getBeams")).get(0),first=call(group,"getFirstChord"),measure=call(first,"getMeasure"),voice=call(first,"getVoice");
            Object other=call(((List<?>)call(sys,"getStacks")).get(1),"getFirstMeasure");
            Object rest=make("sig.inter.RestInter",physical(365,240,8,20,"VERTICAL",null),c("glyph.Shape").getField("QUARTER_REST").get(null),.8,staff,0.);
            Object restChord=make("sig.inter.RestChordInter",.8);call(restChord,"setStaff",staff);call(sig,"addVertex",rest);call(sig,"addVertex",restChord);call(restChord,"addMember",rest);call(same?measure:other,"addInter",restChord);
            call(sig,"addEdge",beam,rest,make("sig.relation.BeamRestRelation"));
            List<Object> before=new ArrayList<>(),after=new ArrayList<>();
            for(Object ch:(List<?>)call(voice,"getChords"))before.add(obj("kind",ch.getClass().getSimpleName(),"bounds",rect(call(ch,"getBounds"))));
            call(first,"justAssignVoice",(Object)null);
            call(group,"setVoice",voice);
            for(Object ch:(List<?>)call(voice,"getChords"))after.add(obj("kind",ch.getClass().getSimpleName(),"bounds",rect(call(ch,"getBounds"))));
            Object rv=call(restChord,"getVoice");
            controls.add(obj("rest_in_same_measure",same,"voice_id",call(voice,"getId"),"voice_measure_number",call(call(call(voice,"getMeasure"),"getStack"),"getIdValue"),"rest_measure_number",call(call(call(restChord,"getMeasure"),"getStack"),"getIdValue"),"rest_voice",rv==null?null:call(rv,"getId"),"voice_chords_before",before,"voice_chords_after",after));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","two declared same/other-measure added RestInter/RestChord with BeamRest on existing original page, first chord Voice field reset via afterReload helper, then BeamGroup.setVoice boundary and stored Voice.addChord order; not automatic rest recognition/rhythm/export","controls",controls));System.exit(0);
    }
}
