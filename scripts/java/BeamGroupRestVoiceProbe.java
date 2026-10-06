// SPDX-License-Identifier: AGPL-3.0-or-later
// Read existing original RHYTHMS saved state; never run recognition/export.
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class BeamGroupRestVoiceProbe extends FoundationOverlapProbe {
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
        Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<Object> groups=new ArrayList<>();
        Object sys0=((List<?>)call(sheet,"getSystems")).get(0),sig0=call(sys0,"getSig"),staff0=call(sys0,"getFirstStaff");
        Object group0=((Collection<?>)call(sig0,"vertexSet")).stream().filter(n->n.getClass().getSimpleName().equals("BeamGroupInter")).findFirst().get();
        Object beam0=((List<?>)call(group0,"getBeams")).get(0),first=call(group0,"getFirstChord"),measure=call(first,"getMeasure");
        Object rest=make("sig.inter.RestInter",physical(365,240,8,20,"VERTICAL",null),c("glyph.Shape").getField("QUARTER_REST").get(null),.8,staff0,0.);
        Object restChord=make("sig.inter.RestChordInter",.8);call(restChord,"setStaff",staff0);call(sig0,"addVertex",rest);call(sig0,"addVertex",restChord);call(restChord,"addMember",rest);call(measure,"addInter",restChord);
        call(sig0,"addEdge",beam0,rest,make("sig.relation.BeamRestRelation"));
        for(Object sys:(Collection<?>)call(sheet,"getSystems"))for(Object group:(Collection<?>)call(call(sys,"getSig"),"vertexSet")){
            if(!group.getClass().getSimpleName().equals("BeamGroupInter")||(Boolean)call(group,"isRemoved"))continue;
            List<Object> beams=new ArrayList<>(),chords=new ArrayList<>();
            for(Object b:(Collection<?>)call(group,"getBeams"))beams.add(obj("kind",b.getClass().getSimpleName(),"bounds",rect(call(b,"getBounds"))));
            for(Object ch:(Collection<?>)call(group,"getChords")){Object voice=call(ch,"getVoice");chords.add(obj("kind",ch.getClass().getSimpleName(),"bounds",rect(call(ch,"getBounds")),"voice",voice==null?null:call(voice,"getId"),"measure_number",call(call(call(ch,"getMeasure"),"getStack"),"getIdValue")));}
            Object voice=call(group,"getVoice");groups.add(obj("system_id",call(sys,"getId"),"bounds",rect(call(group,"getBounds")),"beams",beams,"chords",chords,"voice",voice==null?null:call(voice,"getId"),"voice_measure_number",voice==null?null:call(call(call(voice,"getMeasure"),"getStack"),"getIdValue")));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","declared extra RestInter/RestChord and BeamRestRelation on existing original saved beam group; rest voice unassigned; getter ensemble/order/first-voice control, not raster recognition/export of this rest","groups",groups));System.exit(0);
    }
}
