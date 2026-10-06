// SPDX-License-Identifier: AGPL-3.0-or-later
// Read existing original RHYTHMS saved state; never run recognition/export.
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class BeamGroupVoiceObserver extends FoundationOverlapProbe {
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
        Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<Object> groups=new ArrayList<>();
        for(Object sys:(Collection<?>)call(sheet,"getSystems"))for(Object group:(Collection<?>)call(call(sys,"getSig"),"vertexSet")){
            if(!group.getClass().getSimpleName().equals("BeamGroupInter")||(Boolean)call(group,"isRemoved"))continue;
            List<Object> beams=new ArrayList<>(),chords=new ArrayList<>();
            for(Object b:(Collection<?>)call(group,"getBeams"))beams.add(obj("kind",b.getClass().getSimpleName(),"bounds",rect(call(b,"getBounds"))));
            for(Object ch:(Collection<?>)call(group,"getChords")){Object voice=call(ch,"getVoice");chords.add(obj("kind",ch.getClass().getSimpleName(),"bounds",rect(call(ch,"getBounds")),"voice",voice==null?null:call(voice,"getId"),"measure_number",call(call(call(ch,"getMeasure"),"getStack"),"getIdValue")));}
            Object voice=call(group,"getVoice");groups.add(obj("system_id",call(sys,"getId"),"bounds",rect(call(group,"getBounds")),"beams",beams,"chords",chords,"voice",voice==null?null:call(voice,"getId"),"voice_measure_number",voice==null?null:call(call(call(voice,"getMeasure"),"getStack"),"getIdValue")));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original saved RHYTHMS BeamGroup.getChords/getVoice for existing same-image CLI input; read-only OMR load, distinct from end-to-end transient raster SIG equality","groups",groups));System.exit(0);
    }
}
